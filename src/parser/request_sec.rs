use super::{is_empty, units};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct RequestSec {
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub avg: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub stddev: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub max: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub stddev_percent: f64,
}

impl RequestSec {
    pub fn is_empty(&self) -> bool {
        is_empty::check_f64(&self.avg)
            && is_empty::check_f64(&self.stddev)
            && is_empty::check_f64(&self.max)
            && is_empty::check_f64(&self.stddev_percent)
    }
}

/// parses a line of req/sec from the WRK output.
/// `Req/Sec    56.20k     8.07k   62.00k    86.54%`
impl From<&str> for RequestSec {
    fn from(line: &str) -> Self {
        line.split_whitespace()
            .skip(1)
            .take(4)
            .map(units::parse_count)
            .collect_tuple()
            .map(|(avg, stddev, max, stddev_percent)| RequestSec {
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
    fn parse_thousands() {
        let req = RequestSec::from("Req/Sec    56.20k     8.07k   62.00k    86.54%");
        assert!((req.avg - 56_200.0).abs() < 1.0);
        assert!((req.stddev - 8_070.0).abs() < 1.0);
        assert!((req.max - 62_000.0).abs() < 1.0);
        assert!((req.stddev_percent - 86.54).abs() < 0.01);
    }

    #[test]
    fn parse_plain_numbers() {
        let req = RequestSec::from("Req/Sec   500.00    50.00   800.00    90.00%");
        assert!((req.avg - 500.0).abs() < 0.01);
        assert!((req.stddev - 50.0).abs() < 0.01);
        assert!((req.max - 800.0).abs() < 0.01);
        assert!((req.stddev_percent - 90.0).abs() < 0.01);
    }

    #[test]
    fn parse_empty_line() {
        let req = RequestSec::from("");
        assert!(req.is_empty());
    }

    #[test]
    fn parse_invalid_line() {
        let req = RequestSec::from("garbage data here");
        assert_eq!(req, RequestSec::default());
    }

    #[test]
    fn is_empty_when_all_zero() {
        let req = RequestSec::default();
        assert!(req.is_empty());
    }

    #[test]
    fn is_not_empty_when_any_set() {
        let req = RequestSec {
            avg: 100.0,
            ..Default::default()
        };
        assert!(!req.is_empty());
    }
}
