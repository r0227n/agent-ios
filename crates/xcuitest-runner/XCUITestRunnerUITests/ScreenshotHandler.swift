import XCTest

/// Handles screenshot capture via XCUITest API.
final class ScreenshotHandler {
    /// Capture a full-screen screenshot and return PNG data.
    func captureScreenshot() -> Data? {
        let screenshot = XCUIScreen.main.screenshot()
        return screenshot.pngRepresentation
    }
}
