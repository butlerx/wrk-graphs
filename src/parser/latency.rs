use super::{is_empty, units};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Latency {
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub avg: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub stddev: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub max: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub stddev_percent: f64,
}

impl Latency {
    pub fn is_empty(&self) -> bool {
        is_empty::check_f64(&self.avg)
            && is_empty::check_f64(&self.stddev)
            && is_empty::check_f64(&self.max)
            && is_empty::check_f64(&self.stddev_percent)
    }
}

/// parses a line of latency data from the WRK output.
/// `Latency     6.60ms    1.92ms  12.50ms   68.46%`
impl From<&str> for Latency {
    fn from(line: &str) -> Self {
        line.split_whitespace()
            .skip(1)
            .take(4)
            .map(units::parse_to_milliseconds)
            .collect_tuple()
            .map(|(avg, stddev, max, stddev_percent)| Latency {
                avg,
                stddev,
                max,
                stddev_percent,
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_millisecond_line() {
        let latency = Latency::from("Latency     6.60ms    1.92ms  12.50ms   68.46%");
        assert!((latency.avg - 6.60).abs() < 0.01);
        assert!((latency.stddev - 1.92).abs() < 0.01);
        assert!((latency.max - 12.50).abs() < 0.01);
        assert!((latency.stddev_percent - 68.46).abs() < 0.01);
    }

    #[test]
    fn parse_microsecond_line() {
        let latency = Latency::from("Latency   350.00us   50.00us    1.20ms   75.00%");
        assert!((latency.avg - 0.35).abs() < 0.01);
        assert!((latency.stddev - 0.05).abs() < 0.01);
        assert!((latency.max - 1.20).abs() < 0.01);
        assert!((latency.stddev_percent - 75.0).abs() < 0.01);
    }

    #[test]
    fn parse_seconds_line() {
        let latency = Latency::from("Latency     1.50s   200.00ms    3.00s   90.00%");
        assert!((latency.avg - 1500.0).abs() < 0.01);
        assert!((latency.stddev - 200.0).abs() < 0.01);
        assert!((latency.max - 3000.0).abs() < 0.01);
        assert!((latency.stddev_percent - 90.0).abs() < 0.01);
    }

    #[test]
    fn parse_empty_line() {
        let latency = Latency::from("");
        assert!(latency.is_empty());
    }

    #[test]
    fn parse_invalid_line() {
        let latency = Latency::from("something completely different");
        // Fewer than 4 tokens after skip(1) → default
        assert_eq!(latency, Latency::default());
    }

    #[test]
    fn is_empty_when_all_zero() {
        let latency = Latency::default();
        assert!(latency.is_empty());
    }

    #[test]
    fn is_not_empty_when_any_set() {
        let latency = Latency {
            avg: 1.0,
            ..Default::default()
        };
        assert!(!latency.is_empty());
    }
}
