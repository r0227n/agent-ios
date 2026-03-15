import Foundation
import XCTest

/// Handles app launch and terminate operations.
final class AppHandler {

    /// Launch an app by bundle identifier.
    func launch(bundleIdentifier: String) {
        let app = XCUIApplication(bundleIdentifier: bundleIdentifier)
        app.launch()
    }

    /// Terminate an app by bundle identifier.
    func terminate(bundleIdentifier: String) {
        let app = XCUIApplication(bundleIdentifier: bundleIdentifier)
        app.terminate()
    }
}
