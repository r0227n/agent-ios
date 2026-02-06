import XCTest

/// Handles text input and key press operations via XCUITest API.
final class InputHandler {
    private let app: XCUIApplication

    init(app: XCUIApplication) {
        self.app = app
    }

    /// Type text using the focused element.
    /// Falls back to first responder if no element is focused.
    func typeText(_ text: String) {
        // Use the app's first responder for text input
        // XCUIApplication inherits from XCUIElement, so typeText works on it
        app.typeText(text)
    }

    /// Press a keyboard key by name.
    func keyPress(_ keyName: String) -> Bool {
        switch keyName.lowercased() {
        case "enter", "return":
            app.typeText("\n")
            return true
        case "tab":
            app.typeText("\t")
            return true
        case "delete", "backspace":
            app.typeKey(.delete, modifierFlags: [])
            return true
        case "escape", "esc":
            app.typeKey(.escape, modifierFlags: [])
            return true
        case "space":
            app.typeText(" ")
            return true
        case "up":
            app.typeKey(.upArrow, modifierFlags: [])
            return true
        case "down":
            app.typeKey(.downArrow, modifierFlags: [])
            return true
        case "left":
            app.typeKey(.leftArrow, modifierFlags: [])
            return true
        case "right":
            app.typeKey(.rightArrow, modifierFlags: [])
            return true
        default:
            // Try as a single character
            if keyName.count == 1 {
                app.typeText(keyName)
                return true
            }
            return false
        }
    }
}
