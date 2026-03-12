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

    override func setUp() {
        super.setUp()
        continueAfterFailure = true

        // Use Springboard as the base app to allow cross-app interactions
        app = XCUIApplication(bundleIdentifier: springboardBundleId)
        app.activate()
        activeBundleId = springboardBundleId

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
        ]
        if let udid = ProcessInfo.processInfo.environment["SIMULATOR_UDID"] {
            payload["udid"] = udid
        }
        return payload
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
                        return .ok(["success": true])
                    } else {
                        return .error("Unknown button: \(button). Valid: home, volume_up, volume_down")
                    }
                }, completion: completion)
        }

        // Accessibility tree
        server.get("/accessibility") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            // Parse nested flag from query string (default: true)
            let isNested = !request.path.contains("nested=false")
            // Parse optional depth parameter (e.g. depth=1)
            var maxDepth: Int? = nil
            if let range = request.path.range(of: "depth=") {
                let afterDepth = request.path[range.upperBound...]
                let valueStr = afterDepth.prefix(while: { $0.isNumber })
                maxDepth = Int(valueStr)
            }
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

                    return .ok(["success": true])
                }, completion: completion)
        }

        // Install app
        server.post("/install") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let path = json["path"] as? String
            else {
                return completion(.error("Missing required field: path"))
            }
            self.onMain(
                {
                    self.appHandler.install(path: path)
                }, completion: completion)
        }

        // Uninstall app
        server.post("/uninstall") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                let bundleId = json["bundleId"] as? String
            else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain(
                {
                    self.appHandler.uninstall(bundleId: bundleId)
                }, completion: completion)
        }

        // List installed apps
        server.get("/list-apps") { [weak self] _, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            self.onMain(
                {
                    self.appHandler.listApps()
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
