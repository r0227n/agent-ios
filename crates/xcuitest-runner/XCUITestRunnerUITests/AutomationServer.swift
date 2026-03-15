import CryptoKit
import XCTest

/// Main XCUITest entry point.
/// Starts an HTTP server and routes requests to XCUITest API handlers.
final class AutomationServer: XCTestCase {
    private let springboardBundleId = "com.apple.springboard"
    private var server: HTTPServer!
    private var touchHandler: TouchHandler!
    private var inputHandler: InputHandler!
    private var accessibilityHandler: AccessibilityHandler!
    private var appHandler: AppHandler!
    private var screenshotHandler: ScreenshotHandler!
    private var clipboardHandler: ClipboardHandler!

    /// The target app (Springboard as default - allows controlling any app).
    private var app: XCUIApplication!
    private var activeBundleId: String!
    private var snapshotGeneration: UInt64 = 0
    private var snapshotSequence: UInt64 = 0

    override func setUp() {
        super.setUp()
        continueAfterFailure = true

        // Keep SpringBoard as the neutral coordinate/accessibility context, but
        // do not activate it here. Activating SpringBoard during runner startup
        // causes a visible jump to the Home screen before we restore the target
        // app for the actual command.
        app = XCUIApplication(bundleIdentifier: springboardBundleId)
        activeBundleId = nil

        touchHandler = TouchHandler(app: app)
        inputHandler = InputHandler(app: app)
        accessibilityHandler = AccessibilityHandler(app: app)
        appHandler = AppHandler()
        screenshotHandler = ScreenshotHandler()
        clipboardHandler = ClipboardHandler()

        server = HTTPServer(port: 8200)
        registerRoutes()
    }

    override func tearDown() {
        server?.stop()
        super.tearDown()
    }

    // MARK: - Test Entry Point

    /// Long-running test that keeps the HTTP server alive.
    func testStartAutomationServer() throws {
        try server.start()
        NSLog("[XCUITestRunner] Automation server started on port 8200")

        // Keep the UI test session alive without monopolizing the main queue.
        let keepAlive = expectation(description: "Keep automation server alive")
        wait(for: [keepAlive], timeout: 60 * 60 * 24 * 365)
    }

    // MARK: - Helper to dispatch XCUITest calls on main thread

    private func onMain(_ work: @escaping () -> HTTPResponse, completion: @escaping (HTTPResponse) -> Void) {
        DispatchQueue.main.async {
            let response = work()
            completion(response)
        }
    }

    private func runnerStatus(_ status: String) -> [String: Any] {
        var payload: [String: Any] = [
            "status": status,
            "runner": "xcuitest",
            "snapshot_generation": Int(snapshotGeneration),
        ]
        if let udid = ProcessInfo.processInfo.environment["SIMULATOR_UDID"] {
            payload["udid"] = udid
        }
        if let activeBundleId {
            payload["active_bundle_id"] = activeBundleId
        }
        return payload
    }

    private func advanceSnapshotGeneration() {
        snapshotGeneration += 1
    }

    private func nextSnapshotId() -> String {
        snapshotSequence += 1
        return "snap_\(snapshotGeneration)_\(snapshotSequence)"
    }

    private func queryBool(_ request: HTTPRequest, name: String, defaultValue: Bool) -> Bool {
        guard let value = request.queryValue(name)?.lowercased() else {
            return defaultValue
        }
        switch value {
        case "1", "true", "yes", "on":
            return true
        case "0", "false", "no", "off":
            return false
        default:
            return defaultValue
        }
    }

    private func queryInt(_ request: HTTPRequest, name: String) -> Int? {
        request.queryValue(name).flatMap(Int.init)
    }

    private func sha256Hex(_ data: Data) -> String {
        SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }

    private func switchContext(to bundleId: String) {
        let newApp = XCUIApplication(bundleIdentifier: bundleId)
        let shouldActivate = activeBundleId != bundleId || newApp.state != .runningForeground

        if shouldActivate {
            newApp.activate()
            _ = newApp.wait(for: .runningForeground, timeout: 5)
        }

        app = newApp
        activeBundleId = bundleId
        accessibilityHandler = AccessibilityHandler(app: newApp)
        touchHandler = TouchHandler(app: newApp)
        inputHandler = InputHandler(app: newApp)
    }

    // MARK: - Route Registration

    private func registerRoutes() {
        // Health check (no XCUITest API needed)
        server.get("/health") { _, completion in
            completion(.ok(self.runnerStatus("ok")))
        }

        // Readiness check (must prove XCUITest APIs work on the main queue)
        server.get("/ready") { [weak self] _, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            self.onMain(
                {
                    guard self.accessibilityHandler.isReady() else {
                        return .error("Runner is not ready for accessibility access", status: 500)
                    }
                    return .ok(self.runnerStatus("ready"))
                }, completion: completion)
        }

        // Screenshot
        server.get("/screenshot") { [weak self] _, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            self.onMain(
                {
                    guard let pngData = self.screenshotHandler.captureScreenshot() else {
                        return .error("Failed to capture screenshot", status: 500)
                    }
                    return .data(pngData, contentType: "image/png")
                }, completion: completion)
        }

        // Fast flat snapshot for agent-mobile.
        server.get("/snapshot") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            let depth = self.queryInt(request, name: "depth")
            let interactiveOnly = self.queryBool(request, name: "interactive_only", defaultValue: false)
            let compact = self.queryBool(request, name: "compact", defaultValue: false)
            let visibleOnly = self.queryBool(request, name: "visible_only", defaultValue: true)
            let maxNodes = self.queryInt(request, name: "max_nodes")

            self.onMain(
                {
                    .ok(
                        self.accessibilityHandler.snapshotPayload(
                            snapshotId: self.nextSnapshotId(),
                            snapshotGeneration: self.snapshotGeneration,
                            activeBundleId: self.activeBundleId,
                            maxDepth: depth,
                            interactiveOnly: interactiveOnly,
                            compact: compact,
                            visibleOnly: visibleOnly,
                            maxNodes: maxNodes))
                }, completion: completion)
        }

        // Lightweight UI stability hash.
        server.get("/ui-hash") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            let source = request.queryValue("source")?.lowercased() ?? "screenshot"
            let visibleOnly = self.queryBool(request, name: "visible_only", defaultValue: true)
            let maxDepth = self.queryInt(request, name: "depth") ?? 1

            self.onMain(
                {
                    let digest: String
                    switch source {
                    case "accessibility":
                        digest = self.sha256Hex(
                            Data(
                                self.accessibilityHandler
                                    .snapshotDigestSource(maxDepth: maxDepth, visibleOnly: visibleOnly)
                                    .utf8))
                    default:
                        guard let pngData = self.screenshotHandler.captureScreenshot() else {
                            return .error("Failed to capture screenshot for ui-hash", status: 500)
                        }
                        digest = self.sha256Hex(pngData)
                    }

                    var payload = self.runnerStatus("ok")
                    payload["hash"] = digest
                    payload["source"] = source
                    return .ok(payload)
                }, completion: completion)
        }

        // Fast query for first matching element.
        server.post("/query/first") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let locator = json["locator"] as? String,
                let value = json["value"] as? String
            else {
                return completion(.error("Missing required fields: locator, value"))
            }

            let exact = json["exact"] as? Bool ?? false
            let caseSensitive = json["caseSensitive"] as? Bool ?? false
            let visibleOnly = json["visibleOnly"] as? Bool ?? true
            let maxDepth = json["maxDepth"] as? Int

            self.onMain(
                {
                    var payload = self.runnerStatus("ok")
                    let element = self.accessibilityHandler.queryFirst(
                        locator: locator,
                        value: value,
                        exact: exact,
                        caseSensitive: caseSensitive,
                        visibleOnly: visibleOnly,
                        maxDepth: maxDepth)
                    payload["found"] = element != nil
                    if let element {
                        payload["element"] = element
                    }
                    return .ok(payload)
                }, completion: completion)
        }

        // Fast existence check for matching element.
        server.post("/query/exists") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let locator = json["locator"] as? String,
                let value = json["value"] as? String
            else {
                return completion(.error("Missing required fields: locator, value"))
            }

            let exact = json["exact"] as? Bool ?? false
            let caseSensitive = json["caseSensitive"] as? Bool ?? false
            let visibleOnly = json["visibleOnly"] as? Bool ?? true
            let maxDepth = json["maxDepth"] as? Int

            self.onMain(
                {
                    var payload = self.runnerStatus("ok")
                    payload["exists"] = self.accessibilityHandler.queryExists(
                        locator: locator,
                        value: value,
                        exact: exact,
                        caseSensitive: caseSensitive,
                        visibleOnly: visibleOnly,
                        maxDepth: maxDepth)
                    return .ok(payload)
                }, completion: completion)
        }

        // Tap
        server.post("/tap") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let x = json["x"] as? Double,
                let y = json["y"] as? Double
            else {
                return completion(.error("Missing required fields: x, y"))
            }
            self.onMain(
                {
                    self.touchHandler.tap(x: x, y: y)
                    self.advanceSnapshotGeneration()
                    return .ok(["success": true])
                }, completion: completion)
        }

        // Long press
        server.post("/longpress") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let x = json["x"] as? Double,
                let y = json["y"] as? Double
            else {
                return completion(.error("Missing required fields: x, y"))
            }
            let duration = json["duration"] as? Double ?? 1.0
            self.onMain(
                {
                    self.touchHandler.longPress(x: x, y: y, duration: duration)
                    self.advanceSnapshotGeneration()
                    return .ok(["success": true])
                }, completion: completion)
        }

        // Swipe
        server.post("/swipe") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let startX = json["startX"] as? Double,
                let startY = json["startY"] as? Double,
                let endX = json["endX"] as? Double,
                let endY = json["endY"] as? Double
            else {
                return completion(.error("Missing required fields: startX, startY, endX, endY"))
            }
            let duration = json["duration"] as? Double ?? 0.3
            self.onMain(
                {
                    self.touchHandler.swipe(startX: startX, startY: startY, endX: endX, endY: endY, duration: duration)
                    self.advanceSnapshotGeneration()
                    return .ok(["success": true])
                }, completion: completion)
        }

        // Type text
        server.post("/type") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let text = json["text"] as? String
            else {
                return completion(.error("Missing required field: text"))
            }
            self.onMain(
                {
                    self.inputHandler.typeText(text)
                    self.advanceSnapshotGeneration()
                    return .ok(["success": true])
                }, completion: completion)
        }

        // Key press
        server.post("/keypress") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let key = json["key"] as? String
            else {
                return completion(.error("Missing required field: key"))
            }
            self.onMain(
                {
                    if self.inputHandler.keyPress(key) {
                        self.advanceSnapshotGeneration()
                        return .ok(["success": true])
                    } else {
                        return .error("Unknown key: \(key)")
                    }
                }, completion: completion)
        }

        // Clear text (Select All + Delete)
        server.post("/clear-text") { [weak self] _, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            self.onMain(
                {
                    self.inputHandler.clearText()
                    self.advanceSnapshotGeneration()
                    return .ok(["success": true])
                }, completion: completion)
        }

        // Hardware button press
        server.post("/button") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let button = json["button"] as? String
            else {
                return completion(.error("Missing required field: button"))
            }
            self.onMain(
                {
                    if self.touchHandler.pressButton(button) {
                        self.advanceSnapshotGeneration()
                        return .ok(["success": true])
                    } else {
                        return .error("Unknown button: \(button). Valid: home, volume_up, volume_down")
                    }
                }, completion: completion)
        }

        // Accessibility tree
        server.get("/accessibility") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            let isNested = self.queryBool(request, name: "nested", defaultValue: true)
            let maxDepth = self.queryInt(request, name: "depth")
            self.onMain(
                {
                    let tree = self.accessibilityHandler.getAccessibilityTree(nested: isNested, maxDepth: maxDepth)
                    return .ok(tree)
                }, completion: completion)
        }

        // Launch app
        server.post("/launch") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let bundleId = json["bundleId"] as? String
            else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain(
                {
                    self.appHandler.launch(bundleIdentifier: bundleId)
                    self.switchContext(to: bundleId)
                    self.advanceSnapshotGeneration()

                    return .ok(["success": true, "bundleId": bundleId])
                }, completion: completion)
        }

        // Terminate app
        server.post("/terminate") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let bundleId = json["bundleId"] as? String
            else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain(
                {
                    self.appHandler.terminate(bundleIdentifier: bundleId)
                    self.switchContext(to: self.springboardBundleId)
                    self.advanceSnapshotGeneration()

                    return .ok(["success": true])
                }, completion: completion)
        }

        // Set active app (switch context without launching)
        server.post("/set-app") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let bundleId = json["bundleId"] as? String
            else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain(
                {
                    self.switchContext(to: bundleId)
                    self.advanceSnapshotGeneration()
                    return .ok(["success": true, "bundleId": bundleId])
                }, completion: completion)
        }

        // Clipboard copy
        server.post("/clipboard/copy") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let text = json["text"] as? String
            else {
                return completion(.error("Missing required field: text"))
            }
            self.onMain(
                {
                    self.clipboardHandler.copy(text: text)
                    return .ok(["success": true])
                }, completion: completion)
        }

        // Clipboard paste
        server.get("/clipboard/paste") { [weak self] _, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            self.onMain(
                {
                    let text = self.clipboardHandler.paste()
                    return .ok(["text": text ?? ""])
                }, completion: completion)
        }

        // Clipboard clear
        server.post("/clipboard/clear") { [weak self] _, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            self.onMain(
                {
                    self.clipboardHandler.clear()
                    return .ok(["success": true])
                }, completion: completion)
        }
    }
}
