import UIKit

/// Handles clipboard operations via UIPasteboard.
final class ClipboardHandler {
    /// Copy text to the clipboard.
    func copy(text: String) {
        UIPasteboard.general.string = text
    }

    /// Paste text from the clipboard.
    func paste() -> String? {
        return UIPasteboard.general.string
    }

    /// Clear the clipboard.
    func clear() {
        UIPasteboard.general.strings = []
    }
}
