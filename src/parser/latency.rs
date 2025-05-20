use super::{is_empty, units};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Latency {
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub avg: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub stdev: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub max: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub stdev_percent: f64,
}

impl Latency {
    pub fn is_empty(&self) -> bool {
        is_empty::check_f64(&self.avg)
            && is_empty::check_f64(&self.stdev)
            && is_empty::check_f64(&self.max)
            && is_empty::check_f64(&self.stdev_percent)
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
            .map(|(avg, stdev, max, stdev_percent)| Latency {
                avg,
                stdev,
                max,
                stdev_percent,
            })
            .unwrap_or_default()
    }
}
