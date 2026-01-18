//! Time format parsing utilities

use regex::Regex;

/// Parse formatted time string (e.g., "10s", "5m", "1h", "500ms") to seconds
///
/// Supported units:
/// - ms: milliseconds
/// - s: seconds
/// - m: minutes
/// - h: hours
pub fn formatted_time_to_seconds(
    formatted_time: &str,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let re = Regex::new(r"^([1-9]\d*)(ms|s|m|h)$")?;

    let caps = re.captures(formatted_time).ok_or_else(|| {
        format!(
            "Invalid time format: '{}'. Expected: <time>[ms|s|m|h] (e.g., 10s, 5m, 1h, 500ms)",
            formatted_time
        )
    })?;

    let time: f64 = caps[1].parse()?;
    let unit = &caps[2];

    Ok(match unit {
        "ms" => time / 1000.0,
        "s" => time,
        "m" => time * 60.0,
        "h" => time * 3600.0,
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatted_time_to_seconds() {
        assert!((formatted_time_to_seconds("10s").unwrap() - 10.0).abs() < f64::EPSILON);
        assert!((formatted_time_to_seconds("5m").unwrap() - 300.0).abs() < f64::EPSILON);
        assert!((formatted_time_to_seconds("1h").unwrap() - 3600.0).abs() < f64::EPSILON);
        assert!((formatted_time_to_seconds("500ms").unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_formatted_time_invalid() {
        assert!(formatted_time_to_seconds("10").is_err());
        assert!(formatted_time_to_seconds("abc").is_err());
        assert!(formatted_time_to_seconds("0s").is_err());
        assert!(formatted_time_to_seconds("-5s").is_err());
    }
}
