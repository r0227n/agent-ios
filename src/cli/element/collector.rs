//! Element collection with automatic scrolling.
//!
//! This module provides utilities for collecting all UI elements
//! by automatically scrolling through the screen content.

use std::collections::HashSet;
use std::time::Duration;

use crate::cli::helpers::CommandResult;

use super::FoundElement;

/// Configuration for element collection.
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Maximum number of scroll operations to perform.
    pub max_scrolls: u32,
    /// Delay between scroll operations in milliseconds.
    pub delay_ms: u64,
    /// Screen width for scroll calculations.
    pub screen_width: f64,
    /// Screen height for scroll calculations.
    pub screen_height: f64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            max_scrolls: 10,
            delay_ms: 500,
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
    /// Reached end of scrollable content.
    #[allow(dead_code)]
    EndOfContent,
}

impl std::fmt::Display for CompletionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionReason::MaxScrollsReached => write!(f, "max scrolls reached"),
            CompletionReason::NoNewElements => write!(f, "no new elements found"),
            CompletionReason::EndOfContent => write!(f, "end of content"),
        }
    }
}

/// Generate a unique key for an element (for deduplication).
pub fn element_unique_key(element: &FoundElement) -> String {
    format!(
        "{}|{}|{}",
        element.element_type,
        element.label,
        element.accessibility_id.as_deref().unwrap_or("")
    )
}

/// Merge new elements into existing collection, returning count of new elements.
pub fn merge_elements(
    existing: &mut Vec<FoundElement>,
    new_elements: Vec<FoundElement>,
    seen_keys: &mut HashSet<String>,
) -> usize {
    let mut new_count = 0;

    for element in new_elements {
        let key = element_unique_key(&element);
        if seen_keys.insert(key) {
            existing.push(element);
            new_count += 1;
        }
    }

    new_count
}

/// iOS element collector using gRPC client.
pub struct IosCollector {
    config: CollectorConfig,
}

impl IosCollector {
    pub fn new(config: CollectorConfig) -> Self {
        Self { config }
    }

    /// Collect all elements with automatic scrolling.
    ///
    /// # Arguments
    ///
    /// * `client` - The IDB gRPC client
    /// * `extract_fn` - Function to extract FoundElements from accessibility JSON
    /// * `progress_fn` - Optional callback for progress updates
    pub async fn collect_all<F>(
        &self,
        client: &mut crate::grpc::IdbClient,
        extract_fn: F,
        progress_fn: Option<ProgressCallback>,
    ) -> CommandResult<Vec<FoundElement>>
    where
        F: Fn(&serde_json::Value) -> Vec<FoundElement>,
    {
        let mut all_elements: Vec<FoundElement> = Vec::new();
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
        self.scroll_to_top(client).await?;

        // Get initial elements
        let json_str = client.accessibility_info(None, false).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;
        let initial_elements = extract_fn(&json);
        let initial_count = merge_elements(&mut all_elements, initial_elements, &mut seen_keys);

        if let Some(ref progress) = progress_fn {
            progress(CollectionProgress {
                scrolling_to_top: false,
                scroll_number: 0,
                total_elements: all_elements.len(),
                new_elements: initial_count,
                completed: false,
                completion_reason: None,
            });
        }

        // Scroll and collect
        for scroll_num in 1..=self.config.max_scrolls {
            // Perform scroll down
            self.scroll_down(client).await?;

            // Wait for UI to settle
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            // Get elements after scroll
            let json_str = client.accessibility_info(None, false).await?;
            let json: serde_json::Value = serde_json::from_str(&json_str)?;
            let new_elements = extract_fn(&json);
            let new_count = merge_elements(&mut all_elements, new_elements, &mut seen_keys);

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
                    total_elements: all_elements.len(),
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

    /// Perform a scroll down gesture.
    async fn scroll_down(&self, client: &mut crate::grpc::IdbClient) -> CommandResult<()> {
        use crate::cli::idb::hid::events::swipe_to_events;

        // Scroll from middle-bottom to middle-top (vertical scroll down)
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.7; // Start from 70% down
        let end_y = self.config.screen_height * 0.3; // End at 30% down

        let events = swipe_to_events((center_x, start_y), (center_x, end_y), Some(0.3), None);

        client.hid(events).await?;
        Ok(())
    }

    /// Perform a scroll up gesture (opposite of scroll_down).
    async fn scroll_up(&self, client: &mut crate::grpc::IdbClient) -> CommandResult<()> {
        use crate::cli::idb::hid::events::swipe_to_events;

        // Scroll from top to bottom (swipe downward to scroll content up)
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.3; // Start from 30% down
        let end_y = self.config.screen_height * 0.7; // End at 70% down

        let events = swipe_to_events((center_x, start_y), (center_x, end_y), Some(0.3), None);

        client.hid(events).await?;
        Ok(())
    }

    /// Scroll to the top of the content before collecting.
    ///
    /// Performs repeated scroll-up gestures until reaching the top of the content.
    /// The top is detected when the accessibility tree is unchanged for 2 consecutive scrolls.
    async fn scroll_to_top(&self, client: &mut crate::grpc::IdbClient) -> CommandResult<()> {
        const MAX_SCROLL_UP: u32 = 5;
        let mut prev_snapshot: Option<String> = None;
        let mut consecutive_same = 0;

        for _ in 0..MAX_SCROLL_UP {
            // Scroll up
            self.scroll_up(client).await?;
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            // Get current snapshot (accessibility tree)
            let json_str = client.accessibility_info(None, false).await?;

            // Check if at top (same as previous)
            if let Some(ref prev) = prev_snapshot {
                if &json_str == prev {
                    consecutive_same += 1;
                    if consecutive_same >= 2 {
                        break; // At top
                    }
                } else {
                    consecutive_same = 0;
                }
            }
            prev_snapshot = Some(json_str);
        }
        Ok(())
    }
}

/// Android element collector using ADB.
pub struct AndroidCollector {
    config: CollectorConfig,
    serial: Option<String>,
}

impl AndroidCollector {
    pub fn new(config: CollectorConfig, serial: Option<String>) -> Self {
        Self { config, serial }
    }

    /// Collect all elements with automatic scrolling.
    pub async fn collect_all<F>(
        &self,
        extract_fn: F,
        progress_fn: Option<ProgressCallback>,
    ) -> CommandResult<Vec<FoundElement>>
    where
        F: Fn(
            &[crate::platform::android::adb::uiautomator::AccessibilityElement],
        ) -> Vec<FoundElement>,
    {
        use crate::platform::android::adb::uiautomator;

        let mut all_elements: Vec<FoundElement> = Vec::new();
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
        let elements = uiautomator::parse_ui_hierarchy(&xml)?;
        let initial_elements = extract_fn(&elements);
        let initial_count = merge_elements(&mut all_elements, initial_elements, &mut seen_keys);

        if let Some(ref progress) = progress_fn {
            progress(CollectionProgress {
                scrolling_to_top: false,
                scroll_number: 0,
                total_elements: all_elements.len(),
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
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let new_elements = extract_fn(&elements);
            let new_count = merge_elements(&mut all_elements, new_elements, &mut seen_keys);

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
                    total_elements: all_elements.len(),
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

    /// Perform a scroll down gesture using ADB input.
    async fn scroll_down(&self) -> CommandResult<()> {
        use crate::platform::android::adb::input;

        // Scroll from middle-bottom to middle-top
        let center_x = self.config.screen_width / 2.0;
        let start_y = self.config.screen_height * 0.7;
        let end_y = self.config.screen_height * 0.3;

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

    /// Perform a scroll up gesture (opposite of scroll_down).
    async fn scroll_up(&self) -> CommandResult<()> {
        use crate::platform::android::adb::input;

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
    ///
    /// Performs repeated scroll-up gestures until reaching the top of the content.
    /// The top is detected when the UI hierarchy is unchanged for 2 consecutive scrolls.
    async fn scroll_to_top(&self) -> CommandResult<()> {
        use crate::platform::android::adb::uiautomator;

        const MAX_SCROLL_UP: u32 = 5;
        let mut prev_snapshot: Option<String> = None;
        let mut consecutive_same = 0;

        for _ in 0..MAX_SCROLL_UP {
            // Scroll up
            self.scroll_up().await?;
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;

            // Get current snapshot (UI hierarchy)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_unique_key() {
        let element = FoundElement {
            label: "Test Label".to_string(),
            element_type: "Button".to_string(),
            center: Some((100.0, 200.0)),
            clickable: true,
            scrollable: false,
            frame: None,
            accessibility_id: Some("test_id".to_string()),
            resource_id: None,
        };

        let key = element_unique_key(&element);
        assert_eq!(key, "Button|Test Label|test_id");
    }

    #[test]
    fn test_element_unique_key_no_accessibility_id() {
        let element = FoundElement {
            label: "Test".to_string(),
            element_type: "Label".to_string(),
            center: None,
            clickable: false,
            scrollable: false,
            frame: None,
            accessibility_id: None,
            resource_id: None,
        };

        let key = element_unique_key(&element);
        assert_eq!(key, "Label|Test|");
    }

    #[test]
    fn test_merge_elements_dedup() {
        let mut existing = Vec::new();
        let mut seen = HashSet::new();

        let elements1 = vec![
            FoundElement {
                label: "A".to_string(),
                element_type: "Button".to_string(),
                center: None,
                clickable: true,
                scrollable: false,
                frame: None,
                accessibility_id: None,
                resource_id: None,
            },
            FoundElement {
                label: "B".to_string(),
                element_type: "Button".to_string(),
                center: None,
                clickable: true,
                scrollable: false,
                frame: None,
                accessibility_id: None,
                resource_id: None,
            },
        ];

        let count1 = merge_elements(&mut existing, elements1, &mut seen);
        assert_eq!(count1, 2);
        assert_eq!(existing.len(), 2);

        // Add duplicate and new element
        let elements2 = vec![
            FoundElement {
                label: "A".to_string(), // duplicate
                element_type: "Button".to_string(),
                center: None,
                clickable: true,
                scrollable: false,
                frame: None,
                accessibility_id: None,
                resource_id: None,
            },
            FoundElement {
                label: "C".to_string(), // new
                element_type: "Button".to_string(),
                center: None,
                clickable: true,
                scrollable: false,
                frame: None,
                accessibility_id: None,
                resource_id: None,
            },
        ];

        let count2 = merge_elements(&mut existing, elements2, &mut seen);
        assert_eq!(count2, 1); // Only "C" is new
        assert_eq!(existing.len(), 3);
    }

    #[test]
    fn test_collector_config_default() {
        let config = CollectorConfig::default();
        assert_eq!(config.max_scrolls, 10);
        assert_eq!(config.delay_ms, 500);
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
        assert_eq!(CompletionReason::EndOfContent.to_string(), "end of content");
    }
}
