use super::units;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Latency {
    pub avg: f64,
    pub stdev: f64,
    pub max: f64,
    pub stdev_percent: f64,
}

impl Default for Latency {
    fn default() -> Self {
        Latency {
            avg: 0.0,
            stdev: 0.0,
            max: 0.0,
            stdev_percent: 0.0,
        }
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
