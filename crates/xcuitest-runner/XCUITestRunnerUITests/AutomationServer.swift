import XCTest

/// Main XCUITest entry point.
/// Starts an HTTP server and routes requests to XCUITest API handlers.
final class AutomationServer: XCTestCase {
    private var server: HTTPServer!
    private var touchHandler: TouchHandler!
    private var inputHandler: InputHandler!
    private var accessibilityHandler: AccessibilityHandler!
    private var appHandler: AppHandler!

    /// The target app (Springboard as default - allows controlling any app).
    private var app: XCUIApplication!

    override func setUp() {
        super.setUp()
        continueAfterFailure = true

        // Use Springboard as the base app to allow cross-app interactions
        app = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.activate()

        touchHandler = TouchHandler(app: app)
        inputHandler = InputHandler(app: app)
        accessibilityHandler = AccessibilityHandler(app: app)
        appHandler = AppHandler()

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

        // Keep the test alive using RunLoop so the main thread stays responsive
        while true {
            RunLoop.current.run(mode: .default, before: Date(timeIntervalSinceNow: 1.0))
        }
    }

    // MARK: - Helper to dispatch XCUITest calls on main thread

    private func onMain(_ work: @escaping () -> HTTPResponse, completion: @escaping (HTTPResponse) -> Void) {
        DispatchQueue.main.async {
            let response = work()
            completion(response)
        }
    }

    // MARK: - Route Registration

    private func registerRoutes() {
        // Health check (no XCUITest API needed)
        server.get("/health") { _, completion in
            completion(.ok(["status": "ok", "runner": "xcuitest"]))
        }

        // Tap
        server.post("/tap") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let x = json["x"] as? Double,
                  let y = json["y"] as? Double else {
                return completion(.error("Missing required fields: x, y"))
            }
            self.onMain({
                self.touchHandler.tap(x: x, y: y)
                return .ok(["success": true])
            }, completion: completion)
        }

        // Long press
        server.post("/longpress") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let x = json["x"] as? Double,
                  let y = json["y"] as? Double else {
                return completion(.error("Missing required fields: x, y"))
            }
            let duration = json["duration"] as? Double ?? 1.0
            self.onMain({
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
                  let endY = json["endY"] as? Double else {
                return completion(.error("Missing required fields: startX, startY, endX, endY"))
            }
            let duration = json["duration"] as? Double ?? 0.3
            self.onMain({
                self.touchHandler.swipe(startX: startX, startY: startY, endX: endX, endY: endY, duration: duration)
                return .ok(["success": true])
            }, completion: completion)
        }

        // Type text
        server.post("/type") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let text = json["text"] as? String else {
                return completion(.error("Missing required field: text"))
            }
            self.onMain({
                self.inputHandler.typeText(text)
                return .ok(["success": true])
            }, completion: completion)
        }

        // Key press
        server.post("/keypress") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let key = json["key"] as? String else {
                return completion(.error("Missing required field: key"))
            }
            self.onMain({
                if self.inputHandler.keyPress(key) {
                    return .ok(["success": true])
                } else {
                    return .error("Unknown key: \(key)")
                }
            }, completion: completion)
        }

        // Hardware button press
        server.post("/button") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let button = json["button"] as? String else {
                return completion(.error("Missing required field: button"))
            }
            self.onMain({
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
            self.onMain({
                let tree = self.accessibilityHandler.getAccessibilityTree(nested: isNested)
                return .ok(tree)
            }, completion: completion)
        }

        // Launch app
        server.post("/launch") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let bundleId = json["bundleId"] as? String else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain({
                self.appHandler.launch(bundleIdentifier: bundleId)

                // After launching, update the accessibility handler to use the new app
                let newApp = XCUIApplication(bundleIdentifier: bundleId)
                self.accessibilityHandler = AccessibilityHandler(app: newApp)
                self.touchHandler = TouchHandler(app: newApp)
                self.inputHandler = InputHandler(app: newApp)

                return .ok(["success": true, "bundleId": bundleId])
            }, completion: completion)
        }

        // Terminate app
        server.post("/terminate") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let bundleId = json["bundleId"] as? String else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain({
                self.appHandler.terminate(bundleIdentifier: bundleId)

                // Revert handlers to Springboard
                let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
                self.accessibilityHandler = AccessibilityHandler(app: springboard)
                self.touchHandler = TouchHandler(app: springboard)
                self.inputHandler = InputHandler(app: springboard)

                return .ok(["success": true])
            }, completion: completion)
        }

        // Set active app (switch context without launching)
        server.post("/set-app") { [weak self] request, completion in
            guard let self = self else { return completion(.error("Server unavailable", status: 500)) }
            guard let json = request.json(),
                  let bundleId = json["bundleId"] as? String else {
                return completion(.error("Missing required field: bundleId"))
            }
            self.onMain({
                let newApp = XCUIApplication(bundleIdentifier: bundleId)
                self.accessibilityHandler = AccessibilityHandler(app: newApp)
                self.touchHandler = TouchHandler(app: newApp)
                self.inputHandler = InputHandler(app: newApp)
                return .ok(["success": true, "bundleId": bundleId])
            }, completion: completion)
        }
    }
}
