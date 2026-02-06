import XCTest
import Foundation

/// Handles app launch, terminate, install, uninstall, and list operations.
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

    /// Install an app from a path.
    /// Note: Process (Foundation) is not available on iOS. Install operations
    /// must be performed from the host side using `xcrun simctl install`.
    func install(path: String) -> HTTPResponse {
        return .error("Install is not supported from within the XCUITest runner. Use xcrun simctl install from the host.", status: 501)
    }

    /// Uninstall an app by bundle ID.
    /// Note: Process (Foundation) is not available on iOS. Uninstall operations
    /// must be performed from the host side using `xcrun simctl uninstall`.
    func uninstall(bundleId: String) -> HTTPResponse {
        return .error("Uninstall is not supported from within the XCUITest runner. Use xcrun simctl uninstall from the host.", status: 501)
    }

    /// List installed apps.
    /// Note: Process (Foundation) is not available on iOS. List operations
    /// must be performed from the host side using `xcrun simctl listapps`.
    func listApps() -> HTTPResponse {
        return .error("List apps is not supported from within the XCUITest runner. Use xcrun simctl listapps from the host.", status: 501)
    }
}
