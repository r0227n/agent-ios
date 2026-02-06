import XCTest

/// Handles accessibility tree extraction via XCUITest API.
/// Outputs JSON in idb-compatible format for seamless Rust-side parsing.
final class AccessibilityHandler {
    private let app: XCUIApplication

    init(app: XCUIApplication) {
        self.app = app
    }

    /// Get the accessibility tree as idb-compatible JSON.
    ///
    /// The output format matches idb's accessibility_info response:
    /// ```json
    /// {
    ///   "type": "Application",
    ///   "AXLabel": "MyApp",
    ///   "frame": {"x": 0, "y": 0, "width": 393, "height": 852},
    ///   "enabled": true,
    ///   "children": [...]
    /// }
    /// ```
    func getAccessibilityTree(nested: Bool) -> Any {
        if nested {
            return buildElementTree(from: app)
        } else {
            return buildFlatElement(from: app)
        }
    }

    // MARK: - Tree Building (nested=true)

    private func buildElementTree(from element: XCUIElement) -> [String: Any] {
        var dict = buildElementDict(from: element)

        // Recursively build children
        var children: [[String: Any]] = []
        let childCount = element.children(matching: .any).count
        for i in 0..<childCount {
            let child = element.children(matching: .any).element(boundBy: i)
            if child.exists {
                children.append(buildElementTree(from: child))
            }
        }

        if !children.isEmpty {
            dict["children"] = children
        }

        return dict
    }

    // MARK: - Flat element (nested=false, root only)

    private func buildFlatElement(from element: XCUIElement) -> [String: Any] {
        return buildElementDict(from: element)
    }

    // MARK: - Element Dictionary

    private func buildElementDict(from element: XCUIElement) -> [String: Any] {
        var dict: [String: Any] = [:]

        // Type - map XCUIElement.ElementType to idb type names
        dict["type"] = mapElementType(element.elementType)

        // Label
        let label = element.label
        if !label.isEmpty {
            dict["AXLabel"] = label
        }

        // Value
        if let value = element.value as? String, !value.isEmpty {
            dict["AXValue"] = value
        }

        // Placeholder
        let placeholder = element.placeholderValue
        if let placeholder = placeholder, !placeholder.isEmpty {
            dict["AXPlaceholderValue"] = placeholder
        }

        // Frame
        let frame = element.frame
        dict["frame"] = [
            "x": frame.origin.x,
            "y": frame.origin.y,
            "width": frame.size.width,
            "height": frame.size.height
        ]

        // Enabled
        dict["enabled"] = element.isEnabled

        return dict
    }

    // MARK: - Type Mapping

    /// Map XCUIElement.ElementType to idb-compatible type names.
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
        @unknown default: return "Other"
        }
    }
}
