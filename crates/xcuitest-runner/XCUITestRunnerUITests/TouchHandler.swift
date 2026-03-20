import XCTest

/// Handles tap, long press, and button press operations via XCUITest API.
final class TouchHandler {
    private let app: XCUIApplication
    private let coordinateSpace = XCUIApplication(bundleIdentifier: "com.apple.springboard")

    init(app: XCUIApplication) {
        self.app = app
    }

    /// Tap at absolute screen coordinates.
    func tap(x: Double, y: Double) {
        // Absolute screen coordinates are most stable when anchored to
        // SpringBoard's coordinate space, even while another app is active.
        let normalized = coordinateSpace.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
        let target = normalized.withOffset(CGVector(dx: x, dy: y))
        target.tap()
    }

    /// Long press at absolute screen coordinates.
    func longPress(x: Double, y: Double, duration: Double) {
        let normalized = coordinateSpace.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
        let target = normalized.withOffset(CGVector(dx: x, dy: y))
        target.press(forDuration: duration)
    }

    /// Swipe from one point to another with specified duration.
    func swipe(startX: Double, startY: Double, endX: Double, endY: Double, duration: Double) {
        let normalized = coordinateSpace.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
        let start = normalized.withOffset(CGVector(dx: startX, dy: startY))
        let end = normalized.withOffset(CGVector(dx: endX, dy: endY))

        // velocity = distance / duration (points per second)
        let dx = endX - startX
        let dy = endY - startY
        let distance = sqrt(dx * dx + dy * dy)
        let velocity = max(distance / duration, 50)  // minimum velocity

        start.press(forDuration: 0.05, thenDragTo: end, withVelocity: XCUIGestureVelocity(rawValue: velocity), thenHoldForDuration: 0)
    }

    /// Press a hardware button (home, etc.).
    func pressButton(_ button: String) -> Bool {
        switch button.lowercased() {
        case "home":
            XCUIDevice.shared.press(.home)
            return true
        case "volume_up", "volumeup", "volume-up":
            return false
        case "volume_down", "volumedown", "volume-down":
            return false
        default:
            return false
        }
    }
}
