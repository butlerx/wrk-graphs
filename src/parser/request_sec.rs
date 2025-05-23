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
