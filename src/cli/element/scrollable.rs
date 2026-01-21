//! Scrollable element detection logic.
//!
//! This module provides utilities for determining whether a UI element
//! is scrollable based on its type/class name.

/// iOS scrollable element types.
///
/// These are the UIKit/SwiftUI class names that typically support scrolling.
const IOS_SCROLLABLE_TYPES: &[&str] = &[
    // UIKit classes
    "UIAScrollView",
    "UIATableView",
    "UIACollectionView",
    "UIAWebView",
    "UIATextView",
    "UIScrollView",
    "UITableView",
    "UICollectionView",
    "UIWebView",
    "UITextView",
    // SwiftUI/accessibility role names
    "ScrollView",
    "Table",
    "CollectionView",
    "WebView",
    "List",
    "Grid",
    "TextEditor",
    // Generic scrollable indicators
    "scroll",
    "Scroll",
];

/// Android scrollable element types.
///
/// These are the Android widget class names that typically support scrolling.
const ANDROID_SCROLLABLE_TYPES: &[&str] = &[
    // Core Android widgets
    "android.widget.ScrollView",
    "android.widget.HorizontalScrollView",
    "android.widget.ListView",
    "android.widget.GridView",
    "android.widget.AbsListView",
    "android.widget.ExpandableListView",
    // RecyclerView (most common)
    "android.support.v7.widget.RecyclerView",
    "androidx.recyclerview.widget.RecyclerView",
    // ViewPager
    "android.support.v4.view.ViewPager",
    "androidx.viewpager.widget.ViewPager",
    "androidx.viewpager2.widget.ViewPager2",
    // NestedScrollView
    "android.support.v4.widget.NestedScrollView",
    "androidx.core.widget.NestedScrollView",
    // WebView
    "android.webkit.WebView",
    // Compose
    "LazyColumn",
    "LazyRow",
    "LazyVerticalGrid",
    "LazyHorizontalGrid",
    // Simplified class names (without package)
    "ScrollView",
    "HorizontalScrollView",
    "ListView",
    "GridView",
    "RecyclerView",
    "ViewPager",
    "ViewPager2",
    "NestedScrollView",
    "WebView",
];

/// Determine if an element type indicates scrollability (iOS).
pub fn is_ios_scrollable(element_type: &str) -> bool {
    IOS_SCROLLABLE_TYPES
        .iter()
        .any(|&t| element_type.contains(t) || t.eq_ignore_ascii_case(element_type))
}

/// Determine if an element type indicates scrollability (Android).
pub fn is_android_scrollable(element_type: &str) -> bool {
    ANDROID_SCROLLABLE_TYPES
        .iter()
        .any(|&t| element_type.contains(t) || t.eq_ignore_ascii_case(element_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_scrollable_types() {
        // Standard UIKit types
        assert!(is_ios_scrollable("UIAScrollView"));
        assert!(is_ios_scrollable("UIATableView"));
        assert!(is_ios_scrollable("UIACollectionView"));
        assert!(is_ios_scrollable("UIScrollView"));
        assert!(is_ios_scrollable("UITableView"));

        // SwiftUI types
        assert!(is_ios_scrollable("ScrollView"));
        assert!(is_ios_scrollable("List"));
        assert!(is_ios_scrollable("Table"));

        // Non-scrollable types
        assert!(!is_ios_scrollable("Button"));
        assert!(!is_ios_scrollable("Label"));
        assert!(!is_ios_scrollable("TextField"));
    }

    #[test]
    fn test_android_scrollable_types() {
        // Full class names
        assert!(is_android_scrollable("android.widget.ScrollView"));
        assert!(is_android_scrollable("android.widget.ListView"));
        assert!(is_android_scrollable(
            "androidx.recyclerview.widget.RecyclerView"
        ));

        // Simplified class names
        assert!(is_android_scrollable("ScrollView"));
        assert!(is_android_scrollable("RecyclerView"));
        assert!(is_android_scrollable("ListView"));

        // Non-scrollable types
        assert!(!is_android_scrollable("android.widget.Button"));
        assert!(!is_android_scrollable("TextView"));
        assert!(!is_android_scrollable("ImageView"));
    }

    #[test]
    fn test_partial_match() {
        // Should match partial names
        assert!(is_ios_scrollable("MyCustomScrollView"));
        assert!(is_android_scrollable("com.example.CustomRecyclerView"));
    }
}
