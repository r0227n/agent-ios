//! Snapshot collection with automatic scrolling.
//!
//! This module provides utilities for collecting all UI elements
//! by automatically scrolling through the screen content.

use std::collections::HashSet;
use std::time::Duration;

use crate::helpers::client::CommandResult;
use agent_mobile_core::snapshot::RawElement;
use agent_mobile_platform_android::snapshot::extract_android_elements;
use agent_mobile_platform_ios::snapshot::extract_ios_elements;
use agent_mobile_platform_ios::xcuitest::XCUITestClient;

/// Configuration for snapshot collection with scrolling.
#[derive(Debug, Clone)]
pub struct SnapshotCollectorConfig {
    /// Maximum number of scroll operations to perform.
    pub max_scrolls: u32,
    /// Delay between scroll operations in milliseconds.
    pub delay_ms: u64,
    /// Screen width for scroll calculations.
    pub screen_width: f64,
    /// Screen height for scroll calculations.
    pub screen_height: f64,
}

impl Default for SnapshotCollectorConfig {
    fn default() -> Self {
        Self {
            max_scrolls: 3,
            delay_ms: 100,
            // Default iPhone screen dimensions (will be adjusted per device)
            screen_width: 390.0,
            screen_height: 844.0,
        }
    }
}

/// Progress callback for collection progress updates.
pub type ProgressCallback = Box<dyn Fn(CollectionProgress) + Send>;

/// Progress information during collection.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are consumed by ProgressCallback
pub struct CollectionProgress {
    /// Whether currently scrolling to top before collection.
    pub scrolling_to_top: bool,
    /// Current scroll iteration (1-based).
    pub scroll_number: u32,
    /// Total elements collected so far.
    pub total_elements: usize,
    /// New elements found in this iteration.
    pub new_elements: usize,
    /// Whether collection has completed.
    pub completed: bool,
    /// Reason for completion (if completed).
    pub completion_reason: Option<CompletionReason>,
}

/// Reasons why collection stopped.
#[derive(Debug, Clone)]
pub enum CompletionReason {
    /// Reached maximum scroll count.
    MaxScrollsReached,
    /// No new elements found for consecutive scrolls.
    NoNewElements,
}

impl std::fmt::Display for CompletionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionReason::MaxScrollsReached => write!(f, "max scrolls reached"),
            CompletionReason::NoNewElements => write!(f, "no new elements found"),
        }
    }
}

/// Generate a unique key for an element (for deduplication).
///
/// Uses type, label, and position to uniquely identify elements.
pub fn element_unique_key(element: &RawElement) -> String {
    format!(
        "{}|{}|{:.0},{:.0}",
        element.element_type,
        element.label.as_deref().unwrap_or(""),
        element.frame.x,
        element.frame.y
    )
}

/// Flatten a tree of RawElements into a vector for deduplication.
fn flatten_elements(elements: &[RawElement]) -> Vec<&RawElement> {
    let mut result = Vec::new();
    for element in elements {
        flatten_element_recursive(element, &mut result);
    }
    result
}

fn flatten_element_recursive<'a>(element: &'a RawElement, result: &mut Vec<&'a RawElement>) {
    result.push(element);
    for child in &element.children {
        flatten_element_recursive(child, result);
    }
}

/// Merge new elements into existing tree structure.
///
/// Returns (merged tree, count of new unique elements).
fn merge_element_trees(
    existing: &mut Vec<RawElement>,
    new_elements: Vec<RawElement>,
    seen_keys: &mut HashSet<String>,
) -> usize {
    let mut new_count = 0;

    // Flatten new elements for deduplication check
    let flat_new = flatten_elements(&new_elements);

    for element in flat_new {
        let key = element_unique_key(element);
        if seen_keys.insert(key) {
            new_count += 1;
        }
    }

    // For the first collection, just use the new elements
    if existing.is_empty() {
        *existing = new_elements;
    } else {
        // Merge new elements into existing tree
        // For simplicity, we append new root-level elements that have new unique keys
        for new_elem in new_elements {
            merge_element_into_tree(existing, new_elem, seen_keys);
        }
    }

    new_count
}

/// Merge a single element into the existing tree.
fn merge_element_into_tree(
    tree: &mut Vec<RawElement>,
    element: RawElement,
    _seen_keys: &HashSet<String>,
) {
    // Find if there's an element with matching type and approximate position at root level
    let matching_idx = tree.iter().position(|e| {
        e.element_type == element.element_type
            && (e.frame.x - element.frame.x).abs() < 1.0
            && (e.frame.y - element.frame.y).abs() < 1.0
    });

    if let Some(idx) = matching_idx {
        // Merge children into existing element
        for child in element.children {
            merge_element_into_tree(&mut tree[idx].children, child, _seen_keys);
        }
    } else {
        // Add as new root element
        tree.push(element);
    }
}

/// Snapshot collector using XCUITestClient.
pub struct SnapshotCollector {
    config: SnapshotCollectorConfig,
}

impl SnapshotCollector {
    pub fn new(config: SnapshotCollectorConfig) -> Self {
        Self { config }
    }

    /// Collect all elements with automatic scrolling.
    ///
    /// Returns the merged tree of RawElements from all scroll positions.
    ///
    /// Optimizations:
    /// - If the initial tree contains no scrollable containers, returns immediately.
    /// - Stops after a single scroll that yields no new elements.
    pub async fn collect_all(
        &self,
        client: &XCUITestClient,
        progress_fn: Option<ProgressCallback>,
    ) -> CommandResult<Vec<RawElement>> {
        let mut all_elements: Vec<RawElement> = Vec::new();
        let mut seen_keys: HashSet<String> = HashSet::new();

        // Get initial elements FIRST (before any scrolling)
        let json_str = client.accessibility_info(true).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;
        let initial_elements = extract_ios_elements(&json);
        let initial_count =
            merge_element_trees(&mut all_elements, initial_elements, &mut seen_keys);

        if let Some(ref progress) = progress_fn {
            progress(CollectionProgress {
                scrolling_to_top: false,
                scroll_number: 0,
                total_elements: seen_keys.len(),
                new_elements: initial_count,
                completed: false,
                completion_reason: None,
            });
        }

        // Fast path: if no scrollable containers exist, skip scrolling entirely
        if !has_scrollable_content(&all_elements) {
            if let Some(ref progress) = progress_fn {
                progress(CollectionProgress {
                    scrolling_to_top: false,
                    scroll_number: 0,
                    total_elements: seen_keys.len(),
                    new_elements: 0,
                    completed: true,
                    completion_reason: Some(CompletionReason::NoNewElements),
                });
            }
            return Ok(all_elements);
        }

        // Scroll to top before collecting (only when scrollable content exists)
        if let Some(ref progress) = progress_fn {
            progress(CollectionProgress {
                scrolling_to_top: true,
                scroll_number: 0,
                total_elements: seen_keys.len(),
                new_elements: 0,
                completed: false,
                completion_reason: None,
            });
        }
        self.scroll_to_top(client).await?;

        // Re-fetch after scrolling to top (position may have changed)
        let json_str = client.accessibility_info(true).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;
        let top_elements = extract_ios_elements(&json);
        // Reset and use top-of-page elements as baseline
        all_elements.clear();
        seen_keys.clear();
        merge_element_trees(&mut all_elements, top_elements, &mut seen_keys);

        // Scroll and collect — errors in this loop are non-fatal; we return
        // whatever elements have been collected so far.
        for scroll_num in 1..=self.config.max_scrolls {
            // Perform scroll down
            if self.scroll_down(client).await.is_err() {
                break;
            }

            // Wait for UI to settle
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            // Get elements after scroll (NESTED format)
            let json_str = match client.accessibility_info(true).await {
                Ok(s) => s,
                Err(_) => break,
            };
            let new_elements = match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(json) => extract_ios_elements(&json),
                Err(_) => break,
            };
            let new_count = merge_element_trees(&mut all_elements, new_elements, &mut seen_keys);

            let (completed, reason) = if scroll_num >= self.config.max_scrolls {
                (true, Some(CompletionReason::MaxScrollsReached))
            } else if new_count == 0 {
                // Stop immediately when a single scroll yields no new elements
                (true, Some(CompletionReason::NoNewElements))
            } else {
                (false, None)
            };

            if let Some(ref progress) = progress_fn {
                progress(CollectionProgress {
                    scrolling_to_top: false,
                    scroll_number: scroll_num,
                    total_elements: seen_keys.len(),
                    new_elements: new_count,
                    completed,
                    completion_reason: reason.clone(),
                });
            }

            if completed {
                break;
            }
        }

        Ok(all_elements)
    }

    /// Perform a scroll down gesture via XCUITestClient.
    async fn scroll_down(&self, client: &XCUITestClient) -> CommandResult<()> {
        // Scroll from middle-bottom to middle-top (vertical scroll down)
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.7; // Start from 70% down
        let end_y = self.config.screen_height * 0.3; // End at 30% down

        client
            .swipe((center_x, start_y), (center_x, end_y), 0.3)
            .await?;
        Ok(())
    }

    /// Perform a scroll up gesture via XCUITestClient.
    async fn scroll_up(&self, client: &XCUITestClient) -> CommandResult<()> {
        // Scroll from top to bottom (swipe downward to scroll content up)
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.3; // Start from 30% down
        let end_y = self.config.screen_height * 0.7; // End at 70% down

        client
            .swipe((center_x, start_y), (center_x, end_y), 0.3)
            .await?;
        Ok(())
    }

    /// Scroll to the top of the content before collecting.
    ///
    /// Takes a snapshot **before** scrolling, then compares after each scroll.
    /// If the tree is unchanged after the first scroll, the content is already
    /// at the top and we return immediately (1 iteration instead of 3+).
    ///
    /// Accessibility errors (e.g. timeout) are treated as non-fatal — we simply
    /// stop scrolling and proceed from the current position.
    async fn scroll_to_top(&self, client: &XCUITestClient) -> CommandResult<()> {
        const MAX_SCROLL_UP: u32 = 3;

        // Capture state before any scrolling (shallow depth for speed)
        let mut prev_snapshot = match client.accessibility_info_with_depth(true, Some(1)).await {
            Ok(s) => s,
            Err(_) => return Ok(()), // Runner unresponsive → skip scroll-to-top
        };

        for _ in 0..MAX_SCROLL_UP {
            self.scroll_up(client).await?;
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            let json_str = match client.accessibility_info_with_depth(true, Some(1)).await {
                Ok(s) => s,
                Err(_) => break, // Timeout → stop scrolling
            };
            if json_str == prev_snapshot {
                break; // No change → already at top
            }
            prev_snapshot = json_str;
        }
        Ok(())
    }
}

/// Check if the element tree contains scrollable containers.
///
/// Returns `true` if any element in the tree is a ScrollView, Table,
/// CollectionView, or WebView (which typically support scrolling).
fn has_scrollable_content(elements: &[RawElement]) -> bool {
    elements.iter().any(|e| {
        is_scrollable_type(&e.element_type) || has_scrollable_content_recursive(&e.children)
    })
}

fn has_scrollable_content_recursive(children: &[RawElement]) -> bool {
    children.iter().any(|e| {
        is_scrollable_type(&e.element_type) || has_scrollable_content_recursive(&e.children)
    })
}

fn is_scrollable_type(element_type: &str) -> bool {
    matches!(
        element_type,
        "ScrollView" | "Table" | "CollectionView" | "WebView"
    )
}

/// Android snapshot collector using ADB.
pub struct AndroidSnapshotCollector {
    config: SnapshotCollectorConfig,
    serial: Option<String>,
}

impl AndroidSnapshotCollector {
    pub fn new(config: SnapshotCollectorConfig, serial: Option<String>) -> Self {
        Self { config, serial }
    }

    /// Collect all elements with automatic scrolling.
    ///
    /// Returns the merged tree of RawElements from all scroll positions.
    pub async fn collect_all(
        &self,
        progress_fn: Option<ProgressCallback>,
    ) -> CommandResult<Vec<RawElement>> {
        use agent_mobile_platform_android::adb::uiautomator;

        let mut all_elements: Vec<RawElement> = Vec::new();
        let mut seen_keys: HashSet<String> = HashSet::new();
        let mut consecutive_no_new = 0;

        // Scroll to top first to ensure we start from the beginning
        if let Some(ref progress) = progress_fn {
            progress(CollectionProgress {
                scrolling_to_top: true,
                scroll_number: 0,
                total_elements: 0,
                new_elements: 0,
                completed: false,
                completion_reason: None,
            });
        }
        self.scroll_to_top().await?;

        // Get initial elements
        let xml = uiautomator::dump_ui(self.serial.as_deref()).await?;
        let accessibility_elements = uiautomator::parse_ui_hierarchy(&xml)?;
        let initial_elements = extract_android_elements(&accessibility_elements);
        let initial_count =
            merge_element_trees(&mut all_elements, initial_elements, &mut seen_keys);

        if let Some(ref progress) = progress_fn {
            progress(CollectionProgress {
                scrolling_to_top: false,
                scroll_number: 0,
                total_elements: seen_keys.len(),
                new_elements: initial_count,
                completed: false,
                completion_reason: None,
            });
        }

        // Scroll and collect
        for scroll_num in 1..=self.config.max_scrolls {
            // Perform scroll down
            self.scroll_down().await?;

            // Wait for UI to settle
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            // Get elements after scroll
            let xml = uiautomator::dump_ui(self.serial.as_deref()).await?;
            let accessibility_elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let new_elements = extract_android_elements(&accessibility_elements);
            let new_count = merge_element_trees(&mut all_elements, new_elements, &mut seen_keys);

            // Check for completion
            if new_count == 0 {
                consecutive_no_new += 1;
            } else {
                consecutive_no_new = 0;
            }

            let (completed, reason) = if scroll_num >= self.config.max_scrolls {
                (true, Some(CompletionReason::MaxScrollsReached))
            } else if consecutive_no_new >= 2 {
                (true, Some(CompletionReason::NoNewElements))
            } else {
                (false, None)
            };

            if let Some(ref progress) = progress_fn {
                progress(CollectionProgress {
                    scrolling_to_top: false,
                    scroll_number: scroll_num,
                    total_elements: seen_keys.len(),
                    new_elements: new_count,
                    completed,
                    completion_reason: reason.clone(),
                });
            }

            if completed {
                break;
            }
        }

        Ok(all_elements)
    }

    /// Perform a scroll down gesture using ADB input swipe.
    async fn scroll_down(&self) -> CommandResult<()> {
        use agent_mobile_platform_android::adb::input;

        // Scroll from middle-bottom to middle-top (vertical scroll down)
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.7; // Start from 70% down
        let end_y = self.config.screen_height * 0.3; // End at 30% down

        input::swipe(
            self.serial.as_deref(),
            center_x,
            start_y,
            center_x,
            end_y,
            Some(300), // 300ms duration
        )
        .await?;

        Ok(())
    }

    /// Perform a scroll up gesture using ADB input swipe.
    async fn scroll_up(&self) -> CommandResult<()> {
        use agent_mobile_platform_android::adb::input;

        // Scroll from top to bottom (swipe downward to scroll content up)
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.3; // Start from 30% down
        let end_y = self.config.screen_height * 0.7; // End at 70% down

        input::swipe(
            self.serial.as_deref(),
            center_x,
            start_y,
            center_x,
            end_y,
            Some(300),
        )
        .await?;

        Ok(())
    }

    /// Scroll to the top of the content before collecting.
    async fn scroll_to_top(&self) -> CommandResult<()> {
        use agent_mobile_platform_android::adb::uiautomator;

        const MAX_SCROLL_UP: u32 = 5;
        let mut prev_snapshot: Option<String> = None;
        let mut consecutive_same = 0;

        for _ in 0..MAX_SCROLL_UP {
            // Scroll up
            self.scroll_up().await?;
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            // Get current snapshot (UI dump)
            let xml = uiautomator::dump_ui(self.serial.as_deref()).await?;

            // Check if at top (same as previous)
            if let Some(ref prev) = prev_snapshot {
                if &xml == prev {
                    consecutive_same += 1;
                    if consecutive_same >= 2 {
                        break; // At top
                    }
                } else {
                    consecutive_same = 0;
                }
            }
            prev_snapshot = Some(xml);
        }
        Ok(())
    }
}

/// Get Android screen dimensions.
pub async fn get_android_screen_size(serial: Option<&str>) -> CommandResult<(f64, f64)> {
    use agent_mobile_platform_android::adb::input;

    let (width, height) = input::get_screen_size(serial).await?;
    Ok((width as f64, height as f64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mobile_core::snapshot::Frame;

    fn make_raw(
        element_type: &str,
        label: Option<&str>,
        frame: Frame,
        children: Vec<RawElement>,
    ) -> RawElement {
        RawElement {
            element_type: element_type.to_string(),
            label: label.map(String::from),
            frame,
            enabled: true,
            traits: Vec::new(),
            placeholder: None,
            value: None,
            children,
        }
    }

    #[test]
    fn test_element_unique_key() {
        let element = make_raw(
            "Button",
            Some("Login"),
            Frame {
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 44.0,
            },
            vec![],
        );

        let key = element_unique_key(&element);
        assert_eq!(key, "Button|Login|100,200");
    }

    #[test]
    fn test_element_unique_key_no_label() {
        let element = make_raw(
            "View",
            None,
            Frame {
                x: 50.0,
                y: 100.0,
                width: 300.0,
                height: 400.0,
            },
            vec![],
        );

        let key = element_unique_key(&element);
        assert_eq!(key, "View||50,100");
    }

    #[test]
    fn test_flatten_elements() {
        let tree = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![
                make_raw("Button", Some("A"), Frame::zero(), vec![]),
                make_raw(
                    "View",
                    None,
                    Frame::zero(),
                    vec![make_raw("Button", Some("B"), Frame::zero(), vec![])],
                ),
            ],
        )];

        let flat = flatten_elements(&tree);
        assert_eq!(flat.len(), 4); // Window, Button A, View, Button B
    }

    #[test]
    fn test_merge_element_trees_first_collection() {
        let mut existing = Vec::new();
        let mut seen_keys = HashSet::new();

        let new_elements = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![make_raw("Button", Some("Login"), Frame::zero(), vec![])],
        )];

        let count = merge_element_trees(&mut existing, new_elements, &mut seen_keys);

        assert_eq!(count, 2); // Window + Button
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].children.len(), 1);
    }

    #[test]
    fn test_merge_element_trees_deduplication() {
        let mut existing = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![make_raw("Button", Some("Login"), Frame::zero(), vec![])],
        )];
        let mut seen_keys = HashSet::new();

        // Pre-populate seen keys
        seen_keys.insert("Window|Main|0,0".to_string());
        seen_keys.insert("Button|Login|0,0".to_string());

        // Try to add duplicates
        let new_elements = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![make_raw("Button", Some("Login"), Frame::zero(), vec![])],
        )];

        let count = merge_element_trees(&mut existing, new_elements, &mut seen_keys);

        assert_eq!(count, 0); // No new elements
    }

    #[test]
    fn test_collector_config_default() {
        let config = SnapshotCollectorConfig::default();
        assert_eq!(config.max_scrolls, 3);
        assert_eq!(config.delay_ms, 100);
        assert_eq!(config.screen_width, 390.0);
        assert_eq!(config.screen_height, 844.0);
    }

    #[test]
    fn test_completion_reason_display() {
        assert_eq!(
            CompletionReason::MaxScrollsReached.to_string(),
            "max scrolls reached"
        );
        assert_eq!(
            CompletionReason::NoNewElements.to_string(),
            "no new elements found"
        );
    }

    #[test]
    fn test_has_scrollable_content_false() {
        let elements = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![
                make_raw("Button", Some("Login"), Frame::zero(), vec![]),
                make_raw("StaticText", Some("Hello"), Frame::zero(), vec![]),
            ],
        )];
        assert!(!has_scrollable_content(&elements));
    }

    #[test]
    fn test_has_scrollable_content_true() {
        let elements = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![make_raw(
                "ScrollView",
                None,
                Frame::zero(),
                vec![make_raw("Button", Some("Item"), Frame::zero(), vec![])],
            )],
        )];
        assert!(has_scrollable_content(&elements));
    }

    #[test]
    fn test_has_scrollable_content_nested_table() {
        let elements = vec![make_raw(
            "Window",
            Some("Main"),
            Frame::zero(),
            vec![make_raw(
                "Other",
                None,
                Frame::zero(),
                vec![make_raw("Table", None, Frame::zero(), vec![])],
            )],
        )];
        assert!(has_scrollable_content(&elements));
    }
}
