import Foundation
import XCTest

private struct SnapshotElementModel {
    let elementId: String
    let elementType: String
    let label: String?
    let value: String?
    let placeholder: String?
    let frame: CGRect
    let enabled: Bool
    let interactive: Bool
    let depth: Int?

    var isCompactCandidate: Bool {
        !interactive
            && (label?.isEmpty ?? true)
            && (value?.isEmpty ?? true)
            && (placeholder?.isEmpty ?? true)
    }

    var jsonObject: [String: Any] {
        var payload: [String: Any] = [
            "element_id": elementId,
            "type": elementType,
            "frame": [
                "x": frame.origin.x,
                "y": frame.origin.y,
                "width": frame.size.width,
                "height": frame.size.height,
            ],
            "enabled": enabled,
            "interactive": interactive,
        ]

        if let label = label {
            payload["label"] = label
        }
        if let value = value {
            payload["value"] = value
        }
        if let placeholder = placeholder {
            payload["placeholder"] = placeholder
        }
        if let depth = depth {
            payload["depth"] = depth
        }

        return payload
    }

    func matches(locator: String, value queryValue: String, exact: Bool, caseSensitive: Bool) -> Bool {
        let normalizedQuery = caseSensitive ? queryValue : queryValue.lowercased()

        func normalize(_ value: String?) -> String? {
            guard let value else { return nil }
            return caseSensitive ? value : value.lowercased()
        }

        func matchesValue(_ candidate: String?) -> Bool {
            guard let candidate = normalize(candidate) else { return false }
            return exact ? candidate == normalizedQuery : candidate.contains(normalizedQuery)
        }

        switch locator.lowercased() {
        case "text":
            return matchesValue(label) || matchesValue(value)
        case "label":
            return matchesValue(label)
        case "placeholder":
            return matchesValue(placeholder)
        case "type":
            let candidate = caseSensitive ? elementType : elementType.lowercased()
            return exact ? candidate == normalizedQuery : candidate.contains(normalizedQuery)
        case "element_id":
            return elementId == queryValue
        default:
            return false
        }
    }
}

/// Extracts accessibility information and fast flat snapshots via XCUITest API.
final class AccessibilityHandler {
    private let app: XCUIApplication

    init(app: XCUIApplication) {
        self.app = app
    }

    /// Perform a lightweight probe against the current app context.
    func isReady() -> Bool {
        let state = app.state
        if state != .unknown {
            return true
        }

        if app.windows.firstMatch.exists {
            return true
        }

        if app.otherElements.firstMatch.exists {
            return true
        }

        if app.navigationBars.firstMatch.exists {
            return true
        }

        if app.tabBars.firstMatch.exists {
            return true
        }

        return app.exists
    }

    /// Legacy recursive accessibility tree (kept for compatibility/debug fallback).
    func getAccessibilityTree(nested: Bool, maxDepth: Int? = nil) -> Any {
        if nested {
            return buildElementTree(from: app, currentDepth: 0, maxDepth: maxDepth)
        } else {
            return buildFlatElement(from: app)
        }
    }

    func snapshotPayload(
        snapshotId: String,
        snapshotGeneration: UInt64,
        activeBundleId: String?,
        maxDepth: Int?,
        interactiveOnly: Bool,
        compact: Bool,
        visibleOnly: Bool,
        maxNodes: Int?
    ) -> [String: Any] {
        let elements = collectSnapshotElements(
            maxDepth: maxDepth,
            visibleOnly: visibleOnly,
            maxNodes: maxNodes
        )
        let filtered = filterSnapshotElements(
            elements,
            interactiveOnly: interactiveOnly,
            compact: compact
        )

        var payload: [String: Any] = [
            "snapshot_id": snapshotId,
            "snapshot_generation": Int(snapshotGeneration),
            "elements": filtered.map(\.jsonObject),
        ]
        if let activeBundleId {
            payload["active_bundle_id"] = activeBundleId
        }
        return payload
    }

    func queryFirst(
        locator: String,
        value: String,
        exact: Bool,
        caseSensitive: Bool,
        visibleOnly: Bool,
        maxDepth: Int?
    ) -> [String: Any]? {
        if maxDepth == nil,
            let fastMatch = fastQueryFirst(
                locator: locator,
                value: value,
                exact: exact,
                caseSensitive: caseSensitive,
                visibleOnly: visibleOnly
            )
        {
            return fastMatch.jsonObject
        }

        let elements = collectSnapshotElements(
            maxDepth: maxDepth,
            visibleOnly: visibleOnly,
            maxNodes: nil
        )
        return
            elements
            .first(where: { $0.matches(locator: locator, value: value, exact: exact, caseSensitive: caseSensitive) })?
            .jsonObject
    }

    func queryExists(
        locator: String,
        value: String,
        exact: Bool,
        caseSensitive: Bool,
        visibleOnly: Bool,
        maxDepth: Int?
    ) -> Bool {
        queryFirst(
            locator: locator,
            value: value,
            exact: exact,
            caseSensitive: caseSensitive,
            visibleOnly: visibleOnly,
            maxDepth: maxDepth
        ) != nil
    }

    private func fastQueryFirst(
        locator: String,
        value: String,
        exact: Bool,
        caseSensitive: Bool,
        visibleOnly: Bool
    ) -> SnapshotElementModel? {
        let queryTypes: [XCUIElement.ElementType]
        switch locator.lowercased() {
        case "text":
            queryTypes = [
                .navigationBar,
                .staticText,
                .button,
                .cell,
                .link,
                .textField,
                .secureTextField,
                .searchField,
                .textView,
                .switch,
                .slider,
                .stepper,
                .picker,
                .pickerWheel,
                .datePicker,
                .tab,
            ]
        case "label":
            queryTypes = [
                .navigationBar,
                .staticText,
                .button,
                .cell,
                .link,
                .textField,
                .secureTextField,
                .searchField,
                .textView,
                .tab,
            ]
        case "placeholder":
            queryTypes = [.textField, .secureTextField, .searchField, .textView]
        case "type":
            guard let elementType = elementType(named: value) else { return nil }
            queryTypes = [elementType]
        default:
            return nil
        }

        let screenFrame = currentScreenFrame()
        var seenSignatures: [String: Int] = [:]
        for elementType in queryTypes {
            let elements = app.descendants(matching: elementType).allElementsBoundByIndex
            for element in elements {
                guard
                    let model = snapshotElementModel(
                        for: element,
                        depth: nil,
                        screenFrame: screenFrame,
                        visibleOnly: visibleOnly,
                        seenSignatures: &seenSignatures
                    )
                else {
                    continue
                }

                if model.matches(
                    locator: locator,
                    value: value,
                    exact: exact,
                    caseSensitive: caseSensitive
                ) {
                    return model
                }
            }
        }

        return nil
    }

    func snapshotDigestSource(maxDepth: Int?, visibleOnly: Bool) -> String {
        let elements = collectSnapshotElements(maxDepth: maxDepth, visibleOnly: visibleOnly, maxNodes: 128)
        return
            elements
            .map {
                [
                    $0.elementId,
                    $0.elementType,
                    $0.label ?? "",
                    $0.value ?? "",
                    $0.placeholder ?? "",
                    String(format: "%.0f", $0.frame.origin.x),
                    String(format: "%.0f", $0.frame.origin.y),
                    String(format: "%.0f", $0.frame.size.width),
                    String(format: "%.0f", $0.frame.size.height),
                    $0.enabled ? "1" : "0",
                ].joined(separator: "|")
            }
            .joined(separator: "\n")
    }

    private func collectSnapshotElements(
        maxDepth: Int?,
        visibleOnly: Bool,
        maxNodes: Int?
    ) -> [SnapshotElementModel] {
        let screenFrame = currentScreenFrame()
        var elements: [SnapshotElementModel] = []
        var seenSignatures: [String: Int] = [:]

        if let maxDepth {
            collectRecursiveElements(
                from: app,
                currentDepth: 0,
                maxDepth: maxDepth,
                screenFrame: screenFrame,
                visibleOnly: visibleOnly,
                maxNodes: maxNodes,
                seenSignatures: &seenSignatures,
                elements: &elements
            )
            return elements
        }

        if let root = snapshotElementModel(
            for: app,
            depth: 0,
            screenFrame: screenFrame,
            visibleOnly: false,
            seenSignatures: &seenSignatures
        ) {
            elements.append(root)
        }

        for elementType in snapshotQueryTypes() {
            let descendants = app.descendants(matching: elementType).allElementsBoundByIndex
            for element in descendants {
                if let maxNodes, elements.count >= maxNodes {
                    return elements
                }
                if let model = snapshotElementModel(
                    for: element,
                    depth: nil,
                    screenFrame: screenFrame,
                    visibleOnly: visibleOnly,
                    seenSignatures: &seenSignatures
                ) {
                    elements.append(model)
                }
            }
        }

        return elements
    }

    private func collectRecursiveElements(
        from element: XCUIElement,
        currentDepth: Int,
        maxDepth: Int,
        screenFrame: CGRect,
        visibleOnly: Bool,
        maxNodes: Int?,
        seenSignatures: inout [String: Int],
        elements: inout [SnapshotElementModel]
    ) {
        if let maxNodes, elements.count >= maxNodes {
            return
        }

        let shouldForceInclude = currentDepth == 0
        guard
            let model = snapshotElementModel(
                for: element,
                depth: currentDepth,
                screenFrame: screenFrame,
                visibleOnly: shouldForceInclude ? false : visibleOnly,
                seenSignatures: &seenSignatures
            )
        else {
            return
        }

        elements.append(model)

        if currentDepth >= maxDepth {
            return
        }

        let children = element.children(matching: .any).allElementsBoundByIndex
        for child in children {
            collectRecursiveElements(
                from: child,
                currentDepth: currentDepth + 1,
                maxDepth: maxDepth,
                screenFrame: screenFrame,
                visibleOnly: visibleOnly,
                maxNodes: maxNodes,
                seenSignatures: &seenSignatures,
                elements: &elements
            )
        }
    }

    private func filterSnapshotElements(
        _ elements: [SnapshotElementModel],
        interactiveOnly: Bool,
        compact: Bool
    ) -> [SnapshotElementModel] {
        elements.filter { element in
            if interactiveOnly && !element.interactive && (element.depth ?? 1) != 0 {
                return false
            }
            if compact && element.isCompactCandidate && (element.depth ?? 1) != 0 {
                return false
            }
            return true
        }
    }

    private func snapshotElementModel(
        for element: XCUIElement,
        depth: Int?,
        screenFrame: CGRect,
        visibleOnly: Bool,
        seenSignatures: inout [String: Int]
    ) -> SnapshotElementModel? {
        let elementType = mapElementType(element.elementType)
        if elementType == "Unknown" {
            return nil
        }

        let frame = sanitizedRect(element.frame)
        if visibleOnly && !isVisible(frame: frame, within: screenFrame) {
            return nil
        }

        let label = emptyToNil(element.label)
        let value = normalizedValue(for: element, elementType: elementType)
        let placeholder = normalizedPlaceholder(for: element, elementType: elementType)
        let interactive = isInteractiveType(elementType)
        let elementId = stableElementId(
            elementType: elementType,
            label: label,
            value: value,
            placeholder: placeholder,
            frame: frame,
            seenSignatures: &seenSignatures
        )

        return SnapshotElementModel(
            elementId: elementId,
            elementType: elementType,
            label: label,
            value: value,
            placeholder: placeholder,
            frame: frame,
            enabled: element.isEnabled,
            interactive: interactive,
            depth: depth
        )
    }

    private func snapshotQueryTypes() -> [XCUIElement.ElementType] {
        [
            .navigationBar,
            .tabBar,
            .toolbar,
            .scrollView,
            .table,
            .collectionView,
            .button,
            .link,
            .staticText,
            .textField,
            .secureTextField,
            .searchField,
            .textView,
            .switch,
            .slider,
            .stepper,
            .picker,
            .pickerWheel,
            .datePicker,
            .segmentedControl,
            .tab,
            .alert,
            .sheet,
            .webView,
        ]
    }

    private func elementType(named name: String) -> XCUIElement.ElementType? {
        switch name.lowercased() {
        case "application": return .application
        case "window": return .window
        case "button": return .button
        case "statictext": return .staticText
        case "textfield": return .textField
        case "securetextfield": return .secureTextField
        case "textview": return .textView
        case "image": return .image
        case "scrollview": return .scrollView
        case "table": return .table
        case "cell": return .cell
        case "collectionview": return .collectionView
        case "navigationbar": return .navigationBar
        case "tabbar": return .tabBar
        case "toolbar": return .toolbar
        case "toolbarbutton": return .toolbarButton
        case "switch": return .switch
        case "slider": return .slider
        case "picker": return .picker
        case "pickerwheel": return .pickerWheel
        case "pageindicator": return .pageIndicator
        case "link": return .link
        case "alert": return .alert
        case "sheet": return .sheet
        case "key": return .key
        case "keyboard": return .keyboard
        case "webview": return .webView
        case "map": return .map
        case "group": return .group
        case "other": return .other
        case "icon": return .icon
        case "searchfield": return .searchField
        case "tab": return .tab
        case "segmentedcontrol": return .segmentedControl
        case "stepper": return .stepper
        case "datepicker": return .datePicker
        case "activityindicator": return .activityIndicator
        case "menu": return .menu
        case "menuitem": return .menuItem
        case "menubutton": return .menuButton
        case "progressindicator": return .progressIndicator
        case "toggle": return .toggle
        case "layoutitem": return .layoutItem
        case "handle": return .handle
        case "touchbar": return .touchBar
        case "statusbar": return .statusBar
        case "statusitem": return .statusItem
        case "grid": return .grid
        case "levelindicator": return .levelIndicator
        case "layoutarea": return .layoutArea
        case "outline": return .outline
        case "outlinerow": return .outlineRow
        case "browser": return .browser
        case "incrementarrow": return .incrementArrow
        case "decrementarrow": return .decrementArrow
        case "colorwell": return .colorWell
        case "combobox": return .comboBox
        case "disclosuretriangle": return .disclosureTriangle
        case "drawer": return .drawer
        case "menubar": return .menuBar
        case "menubaritem": return .menuBarItem
        case "popupbutton": return .popUpButton
        case "popover": return .popover
        case "radiobutton": return .radioButton
        case "radiogroup": return .radioGroup
        case "scrollbar": return .scrollBar
        case "splitgroup": return .splitGroup
        case "splitter": return .splitter
        case "tabgroup": return .tabGroup
        case "tablecolumn": return .tableColumn
        case "tablerow": return .tableRow
        case "dialog": return .dialog
        case "checkbox": return .checkBox
        case "ratingindicator": return .ratingIndicator
        default: return nil
        }
    }

    private func stableElementId(
        elementType: String,
        label: String?,
        value: String?,
        placeholder: String?,
        frame: CGRect,
        seenSignatures: inout [String: Int]
    ) -> String {
        let rounded = [
            Int(frame.origin.x.rounded()),
            Int(frame.origin.y.rounded()),
            Int(frame.size.width.rounded()),
            Int(frame.size.height.rounded()),
        ]
        let base = [
            elementType,
            label ?? "",
            value ?? "",
            placeholder ?? "",
            rounded.map(String.init).joined(separator: ","),
        ]
        .map { $0.replacingOccurrences(of: "|", with: "\\|") }
        .joined(separator: "|")

        let ordinal = (seenSignatures[base] ?? 0) + 1
        seenSignatures[base] = ordinal
        return ordinal == 1 ? base : "\(base)#\(ordinal)"
    }

    private func normalizedValue(for element: XCUIElement, elementType: String) -> String? {
        let shouldReadValue = matches(
            elementType,
            "TextField",
            "SecureTextField",
            "SearchField",
            "TextView",
            "Switch",
            "Slider",
            "Stepper",
            "Picker",
            "DatePicker"
        )

        guard shouldReadValue else { return nil }
        return stringifyValue(element.value)
    }

    private func normalizedPlaceholder(for element: XCUIElement, elementType: String) -> String? {
        guard matches(elementType, "TextField", "SecureTextField", "SearchField", "TextView") else {
            return nil
        }
        return emptyToNil(element.placeholderValue)
    }

    private func currentScreenFrame() -> CGRect {
        let appFrame = sanitizedRect(app.frame)
        if appFrame.size.width > 0, appFrame.size.height > 0 {
            return appFrame
        }

        let windowFrame = sanitizedRect(app.windows.firstMatch.frame)
        if windowFrame.size.width > 0, windowFrame.size.height > 0 {
            return windowFrame
        }

        return CGRect(x: 0, y: 0, width: 393, height: 852)
    }

    private func isVisible(frame: CGRect, within screenFrame: CGRect) -> Bool {
        guard frame.size.width > 0, frame.size.height > 0 else { return false }
        guard frame.origin.x.isFinite, frame.origin.y.isFinite else { return false }
        return frame.intersects(screenFrame.insetBy(dx: -1, dy: -1))
    }

    private func sanitizedRect(_ rect: CGRect) -> CGRect {
        CGRect(
            x: rect.origin.x.isFinite ? rect.origin.x : 0,
            y: rect.origin.y.isFinite ? rect.origin.y : 0,
            width: rect.size.width.isFinite ? rect.size.width : 0,
            height: rect.size.height.isFinite ? rect.size.height : 0
        )
    }

    private func stringifyValue(_ rawValue: Any?) -> String? {
        if let string = rawValue as? String {
            return emptyToNil(string)
        }
        if let number = rawValue as? NSNumber {
            return number.stringValue
        }
        return rawValue.map { String(describing: $0) }
            .flatMap(emptyToNil)
    }

    private func emptyToNil(_ value: String?) -> String? {
        guard let value else { return nil }
        return value.isEmpty ? nil : value
    }

    private func matches(_ value: String, _ candidates: String...) -> Bool {
        candidates.contains(value)
    }

    private func isInteractiveType(_ type: String) -> Bool {
        matches(
            type,
            "Button",
            "Link",
            "TextField",
            "SecureTextField",
            "SearchField",
            "TextView",
            "Switch",
            "Slider",
            "Stepper",
            "Picker",
            "DatePicker",
            "SegmentedControl",
            "Tab",
            "TabBar",
            "MenuItem",
            "MenuButton",
            "PopUpButton",
            "ComboBox",
            "DisclosureTriangle",
            "CheckBox",
            "RadioButton",
            "IncrementArrow",
            "DecrementArrow",
            "Cell"
        )
    }

    // MARK: - Legacy tree building

    private func buildElementTree(from element: XCUIElement, currentDepth: Int, maxDepth: Int?) -> [String: Any] {
        var dict = buildElementDict(from: element)

        if let maxDepth, currentDepth >= maxDepth {
            return dict
        }

        let childElements = element.children(matching: .any).allElementsBoundByIndex
        if !childElements.isEmpty {
            var children: [[String: Any]] = []
            children.reserveCapacity(childElements.count)
            for child in childElements {
                children.append(buildElementTree(from: child, currentDepth: currentDepth + 1, maxDepth: maxDepth))
            }
            dict["children"] = children
        }

        return dict
    }

    private func buildFlatElement(from element: XCUIElement) -> [String: Any] {
        buildElementDict(from: element)
    }

    private func buildElementDict(from element: XCUIElement) -> [String: Any] {
        var dict: [String: Any] = [:]

        let elementType = mapElementType(element.elementType)
        dict["type"] = elementType

        if let label = emptyToNil(element.label) {
            dict["AXLabel"] = label
        }

        if let value = stringifyValue(element.value) {
            dict["AXValue"] = value
        }

        if let placeholder = emptyToNil(element.placeholderValue) {
            dict["AXPlaceholderValue"] = placeholder
        }

        let frame = sanitizedRect(element.frame)
        dict["frame"] = [
            "x": frame.origin.x,
            "y": frame.origin.y,
            "width": frame.size.width,
            "height": frame.size.height,
        ]

        dict["enabled"] = element.isEnabled
        return dict
    }

    private func mapElementType(_ type: XCUIElement.ElementType) -> String {
        switch type {
        case .application: return "Application"
        case .window: return "Window"
        case .button: return "Button"
        case .staticText: return "StaticText"
        case .textField: return "TextField"
        case .secureTextField: return "SecureTextField"
        case .textView: return "TextView"
        case .image: return "Image"
        case .scrollView: return "ScrollView"
        case .table: return "Table"
        case .cell: return "Cell"
        case .collectionView: return "CollectionView"
        case .navigationBar: return "NavigationBar"
        case .tabBar: return "TabBar"
        case .toolbar: return "Toolbar"
        case .toolbarButton: return "ToolbarButton"
        case .switch: return "Switch"
        case .slider: return "Slider"
        case .picker: return "Picker"
        case .pageIndicator: return "PageIndicator"
        case .link: return "Link"
        case .alert: return "Alert"
        case .sheet: return "Sheet"
        case .key: return "Key"
        case .keyboard: return "Keyboard"
        case .webView: return "WebView"
        case .map: return "Map"
        case .group: return "Group"
        case .other: return "Other"
        case .icon: return "Icon"
        case .searchField: return "SearchField"
        case .tab: return "Tab"
        case .segmentedControl: return "SegmentedControl"
        case .stepper: return "Stepper"
        case .datePicker: return "DatePicker"
        case .activityIndicator: return "ActivityIndicator"
        case .menu: return "Menu"
        case .menuItem: return "MenuItem"
        case .menuButton: return "MenuButton"
        case .progressIndicator: return "ProgressIndicator"
        case .toggle: return "Toggle"
        case .layoutItem: return "LayoutItem"
        case .handle: return "Handle"
        case .touchBar: return "TouchBar"
        case .statusBar: return "StatusBar"
        case .statusItem: return "StatusItem"
        case .rulerMarker: return "RulerMarker"
        case .grid: return "Grid"
        case .levelIndicator: return "LevelIndicator"
        case .layoutArea: return "LayoutArea"
        case .any: return "Any"
        case .outline: return "Outline"
        case .outlineRow: return "OutlineRow"
        case .browser: return "Browser"
        case .incrementArrow: return "IncrementArrow"
        case .decrementArrow: return "DecrementArrow"
        case .colorWell: return "ColorWell"
        case .comboBox: return "ComboBox"
        case .disclosureTriangle: return "DisclosureTriangle"
        case .dockItem: return "DockItem"
        case .drawer: return "Drawer"
        case .helpTag: return "HelpTag"
        case .matte: return "Matte"
        case .menuBar: return "MenuBar"
        case .menuBarItem: return "MenuBarItem"
        case .popUpButton: return "PopUpButton"
        case .popover: return "Popover"
        case .radioButton: return "RadioButton"
        case .radioGroup: return "RadioGroup"
        case .relevanceIndicator: return "RelevanceIndicator"
        case .ruler: return "Ruler"
        case .scrollBar: return "ScrollBar"
        case .splitGroup: return "SplitGroup"
        case .splitter: return "Splitter"
        case .tabGroup: return "TabGroup"
        case .tableColumn: return "TableColumn"
        case .tableRow: return "TableRow"
        case .timeline: return "Timeline"
        case .valueIndicator: return "ValueIndicator"
        case .dialog: return "Dialog"
        case .checkBox: return "CheckBox"
        case .pickerWheel: return "PickerWheel"
        case .ratingIndicator: return "RatingIndicator"
        default: return "Other"
        }
    }
}
