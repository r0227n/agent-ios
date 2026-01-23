use crate::proto::idb::{
    hid_event::{self, Event, HidButtonType, HidDirection},
    HidEvent, Point,
};

/// Convert tap coordinates to HID events
pub fn tap_to_events(x: f64, y: f64, duration: Option<f64>) -> Vec<HidEvent> {
    let touch = hid_event::HidTouch {
        point: Some(Point { x, y }),
    };
    let action = hid_event::HidPressAction {
        action: Some(hid_event::hid_press_action::Action::Touch(touch)),
    };
    press_with_duration(action, duration)
}

/// Convert button press to HID events
pub fn button_to_events(button_type: HidButtonType, duration: Option<f64>) -> Vec<HidEvent> {
    let button = hid_event::HidButton {
        button: button_type as i32,
    };
    let action = hid_event::HidPressAction {
        action: Some(hid_event::hid_press_action::Action::Button(button)),
    };
    press_with_duration(action, duration)
}

/// Convert key press to HID events
pub fn key_to_events(keycode: u64, duration: Option<f64>) -> Vec<HidEvent> {
    let key = hid_event::HidKey { keycode };
    let action = hid_event::HidPressAction {
        action: Some(hid_event::hid_press_action::Action::Key(key)),
    };
    press_with_duration(action, duration)
}

/// Convert key sequence to HID events
pub fn key_sequence_to_events(key_sequence: Vec<u64>) -> Vec<HidEvent> {
    let mut events = Vec::new();
    for keycode in key_sequence {
        events.extend(key_to_events(keycode, None));
    }
    events
}

/// Convert swipe gesture to HID events
pub fn swipe_to_events(
    start: (f64, f64),
    end: (f64, f64),
    duration: Option<f64>,
    delta: Option<f64>,
) -> Vec<HidEvent> {
    let swipe = hid_event::HidSwipe {
        start: Some(Point {
            x: start.0,
            y: start.1,
        }),
        end: Some(Point { x: end.0, y: end.1 }),
        delta: delta.unwrap_or(0.0),
        duration: duration.unwrap_or(0.0),
    };
    vec![HidEvent {
        event: Some(Event::Swipe(swipe)),
    }]
}

/// Convert text to HID events with character mapping
pub fn text_to_events(text: &str) -> Result<Vec<HidEvent>, String> {
    let mut events = Vec::new();
    for ch in text.chars() {
        match char_to_events(ch) {
            Some(ch_events) => events.extend(ch_events),
            None => return Err(format!("No keycode found for character: {}", ch)),
        }
    }
    Ok(events)
}

/// Helper: Create DOWN, optional DELAY, UP sequence
fn press_with_duration(action: hid_event::HidPressAction, duration: Option<f64>) -> Vec<HidEvent> {
    let mut events = Vec::new();

    // DOWN event
    events.push(HidEvent {
        event: Some(Event::Press(hid_event::HidPress {
            action: Some(action),
            direction: HidDirection::Down as i32,
        })),
    });

    // Optional DELAY
    if let Some(d) = duration {
        events.push(HidEvent {
            event: Some(Event::Delay(hid_event::HidDelay { duration: d })),
        });
    }

    // UP event
    events.push(HidEvent {
        event: Some(Event::Press(hid_event::HidPress {
            action: Some(action),
            direction: HidDirection::Up as i32,
        })),
    });

    events
}

/// Helper: Create key down event
fn key_down(keycode: u64) -> HidEvent {
    let key = hid_event::HidKey { keycode };
    let action = hid_event::HidPressAction {
        action: Some(hid_event::hid_press_action::Action::Key(key)),
    };
    HidEvent {
        event: Some(Event::Press(hid_event::HidPress {
            action: Some(action),
            direction: HidDirection::Down as i32,
        })),
    }
}

/// Helper: Create key up event
fn key_up(keycode: u64) -> HidEvent {
    let key = hid_event::HidKey { keycode };
    let action = hid_event::HidPressAction {
        action: Some(hid_event::hid_press_action::Action::Key(key)),
    };
    HidEvent {
        event: Some(Event::Press(hid_event::HidPress {
            action: Some(action),
            direction: HidDirection::Up as i32,
        })),
    }
}

/// Helper: Create shifted key press (shift + key)
fn key_press_shifted(keycode: u64) -> Vec<HidEvent> {
    vec![
        key_down(225), // Left shift
        key_down(keycode),
        key_up(keycode),
        key_up(225),
    ]
}

/// Character to keycode mapping (implements Python KEY_MAP)
fn char_to_events(ch: char) -> Option<Vec<HidEvent>> {
    match ch {
        // Lowercase letters (a-z) -> keycodes 4-29
        'a' => Some(key_to_events(4, None)),
        'b' => Some(key_to_events(5, None)),
        'c' => Some(key_to_events(6, None)),
        'd' => Some(key_to_events(7, None)),
        'e' => Some(key_to_events(8, None)),
        'f' => Some(key_to_events(9, None)),
        'g' => Some(key_to_events(10, None)),
        'h' => Some(key_to_events(11, None)),
        'i' => Some(key_to_events(12, None)),
        'j' => Some(key_to_events(13, None)),
        'k' => Some(key_to_events(14, None)),
        'l' => Some(key_to_events(15, None)),
        'm' => Some(key_to_events(16, None)),
        'n' => Some(key_to_events(17, None)),
        'o' => Some(key_to_events(18, None)),
        'p' => Some(key_to_events(19, None)),
        'q' => Some(key_to_events(20, None)),
        'r' => Some(key_to_events(21, None)),
        's' => Some(key_to_events(22, None)),
        't' => Some(key_to_events(23, None)),
        'u' => Some(key_to_events(24, None)),
        'v' => Some(key_to_events(25, None)),
        'w' => Some(key_to_events(26, None)),
        'x' => Some(key_to_events(27, None)),
        'y' => Some(key_to_events(28, None)),
        'z' => Some(key_to_events(29, None)),
        // Uppercase letters (A-Z) -> shifted keycodes 4-29
        'A' => Some(key_press_shifted(4)),
        'B' => Some(key_press_shifted(5)),
        'C' => Some(key_press_shifted(6)),
        'D' => Some(key_press_shifted(7)),
        'E' => Some(key_press_shifted(8)),
        'F' => Some(key_press_shifted(9)),
        'G' => Some(key_press_shifted(10)),
        'H' => Some(key_press_shifted(11)),
        'I' => Some(key_press_shifted(12)),
        'J' => Some(key_press_shifted(13)),
        'K' => Some(key_press_shifted(14)),
        'L' => Some(key_press_shifted(15)),
        'M' => Some(key_press_shifted(16)),
        'N' => Some(key_press_shifted(17)),
        'O' => Some(key_press_shifted(18)),
        'P' => Some(key_press_shifted(19)),
        'Q' => Some(key_press_shifted(20)),
        'R' => Some(key_press_shifted(21)),
        'S' => Some(key_press_shifted(22)),
        'T' => Some(key_press_shifted(23)),
        'U' => Some(key_press_shifted(24)),
        'V' => Some(key_press_shifted(25)),
        'W' => Some(key_press_shifted(26)),
        'X' => Some(key_press_shifted(27)),
        'Y' => Some(key_press_shifted(28)),
        'Z' => Some(key_press_shifted(29)),
        // Numbers (0-9) -> keycodes 30-39
        '1' => Some(key_to_events(30, None)),
        '2' => Some(key_to_events(31, None)),
        '3' => Some(key_to_events(32, None)),
        '4' => Some(key_to_events(33, None)),
        '5' => Some(key_to_events(34, None)),
        '6' => Some(key_to_events(35, None)),
        '7' => Some(key_to_events(36, None)),
        '8' => Some(key_to_events(37, None)),
        '9' => Some(key_to_events(38, None)),
        '0' => Some(key_to_events(39, None)),
        // Special characters
        '\n' => Some(key_to_events(40, None)), // Enter
        ' ' => Some(key_to_events(44, None)),  // Space
        '-' => Some(key_to_events(45, None)),
        '=' => Some(key_to_events(46, None)),
        '[' => Some(key_to_events(47, None)),
        ']' => Some(key_to_events(48, None)),
        '\\' => Some(key_to_events(49, None)),
        ';' => Some(key_to_events(51, None)),
        '\'' => Some(key_to_events(52, None)),
        '`' => Some(key_to_events(53, None)),
        ',' => Some(key_to_events(54, None)),
        '.' => Some(key_to_events(55, None)),
        '/' => Some(key_to_events(56, None)),
        // Shifted special characters
        '!' => Some(key_press_shifted(30)), // Shift + 1
        '@' => Some(key_press_shifted(31)), // Shift + 2
        '#' => Some(key_press_shifted(32)), // Shift + 3
        '$' => Some(key_press_shifted(33)), // Shift + 4
        '%' => Some(key_press_shifted(34)), // Shift + 5
        '^' => Some(key_press_shifted(35)), // Shift + 6
        '&' => Some(key_press_shifted(36)), // Shift + 7
        '*' => Some(key_press_shifted(37)), // Shift + 8
        '(' => Some(key_press_shifted(38)), // Shift + 9
        ')' => Some(key_press_shifted(39)), // Shift + 0
        '_' => Some(key_press_shifted(45)), // Shift + -
        '+' => Some(key_press_shifted(46)), // Shift + =
        '{' => Some(key_press_shifted(47)), // Shift + [
        '}' => Some(key_press_shifted(48)), // Shift + ]
        '|' => Some(key_press_shifted(49)), // Shift + \
        ':' => Some(key_press_shifted(51)), // Shift + ;
        '"' => Some(key_press_shifted(52)), // Shift + '
        '~' => Some(key_press_shifted(53)), // Shift + `
        '<' => Some(key_press_shifted(54)), // Shift + ,
        '>' => Some(key_press_shifted(55)), // Shift + .
        '?' => Some(key_press_shifted(56)), // Shift + /
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::idb::hid_event::{HidButtonType, HidDirection};

    #[test]
    fn test_tap_to_events_without_duration() {
        let events = tap_to_events(100.0, 200.0, None);
        assert_eq!(events.len(), 2); // DOWN + UP

        // DOWN イベント確認
        if let Some(Event::Press(press)) = &events[0].event {
            assert_eq!(press.direction, HidDirection::Down as i32);
            if let Some(hid_event::hid_press_action::Action::Touch(touch)) =
                &press.action.as_ref().unwrap().action
            {
                assert_eq!(touch.point.as_ref().unwrap().x, 100.0);
                assert_eq!(touch.point.as_ref().unwrap().y, 200.0);
            } else {
                panic!("Expected Touch action");
            }
        } else {
            panic!("Expected Press event");
        }

        // UP イベント確認
        if let Some(Event::Press(press)) = &events[1].event {
            assert_eq!(press.direction, HidDirection::Up as i32);
        }
    }

    #[test]
    fn test_tap_to_events_with_duration() {
        let events = tap_to_events(100.0, 200.0, Some(0.5));
        assert_eq!(events.len(), 3); // DOWN + DELAY + UP

        // DELAY イベント確認
        if let Some(Event::Delay(delay)) = &events[1].event {
            assert_eq!(delay.duration, 0.5);
        } else {
            panic!("Expected Delay event");
        }
    }

    #[test]
    fn test_button_to_events() {
        let events = button_to_events(HidButtonType::Home, None);
        assert_eq!(events.len(), 2); // DOWN + UP

        // ボタンタイプ確認
        if let Some(Event::Press(press)) = &events[0].event {
            if let Some(hid_event::hid_press_action::Action::Button(button)) =
                &press.action.as_ref().unwrap().action
            {
                assert_eq!(button.button, HidButtonType::Home as i32);
            }
        }
    }

    #[test]
    fn test_key_to_events() {
        let events = key_to_events(40, None); // Enter key
        assert_eq!(events.len(), 2);

        // キーコード確認
        if let Some(Event::Press(press)) = &events[0].event {
            if let Some(hid_event::hid_press_action::Action::Key(key)) =
                &press.action.as_ref().unwrap().action
            {
                assert_eq!(key.keycode, 40);
            }
        }
    }

    #[test]
    fn test_key_sequence_to_events() {
        let events = key_sequence_to_events(vec![4, 5, 6]); // a, b, c
        assert_eq!(events.len(), 6); // 各キー DOWN + UP = 3 * 2
    }

    #[test]
    fn test_text_to_events_lowercase() {
        let events = text_to_events("abc").unwrap();
        assert_eq!(events.len(), 6); // a, b, c 各2イベント
    }

    #[test]
    fn test_text_to_events_uppercase() {
        let events = text_to_events("A").unwrap();
        assert_eq!(events.len(), 4); // Shift DOWN, A DOWN, A UP, Shift UP

        // Shiftキー(keycode 225)確認
        if let Some(Event::Press(press)) = &events[0].event {
            if let Some(hid_event::hid_press_action::Action::Key(key)) =
                &press.action.as_ref().unwrap().action
            {
                assert_eq!(key.keycode, 225); // Left shift
            }
        }
    }

    #[test]
    fn test_text_to_events_with_numbers() {
        let events = text_to_events("123").unwrap();
        assert_eq!(events.len(), 6); // 3文字 × 2イベント
    }

    #[test]
    fn test_text_to_events_with_special_chars() {
        let events = text_to_events("!@#").unwrap();
        assert_eq!(events.len(), 12); // 3文字 × 4イベント (各文字 shift down/up + key down/up)
    }

    #[test]
    fn test_text_to_events_invalid_char() {
        let result = text_to_events("あ"); // 日本語文字
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No keycode found"));
    }

    #[test]
    fn test_swipe_to_events() {
        let events = swipe_to_events((100.0, 100.0), (200.0, 200.0), Some(0.5), Some(10.0));
        assert_eq!(events.len(), 1); // HIDSwipe イベント1つ

        // スワイプ内容確認
        if let Some(Event::Swipe(swipe)) = &events[0].event {
            assert_eq!(swipe.start.as_ref().unwrap().x, 100.0);
            assert_eq!(swipe.start.as_ref().unwrap().y, 100.0);
            assert_eq!(swipe.end.as_ref().unwrap().x, 200.0);
            assert_eq!(swipe.end.as_ref().unwrap().y, 200.0);
            assert_eq!(swipe.duration, 0.5);
            assert_eq!(swipe.delta, 10.0);
        } else {
            panic!("Expected Swipe event");
        }
    }

    #[test]
    fn test_swipe_to_events_defaults() {
        let events = swipe_to_events((0.0, 0.0), (100.0, 100.0), None, None);
        assert_eq!(events.len(), 1);

        // デフォルト値確認
        if let Some(Event::Swipe(swipe)) = &events[0].event {
            assert_eq!(swipe.duration, 0.0);
            assert_eq!(swipe.delta, 0.0);
        }
    }
}
