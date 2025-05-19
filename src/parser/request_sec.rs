use super::units;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RequestSec {
    pub avg: f64,
    pub stdev: f64,
    pub max: f64,
    pub stdev_percent: f64,
}

impl Default for RequestSec {
    fn default() -> Self {
        RequestSec {
            avg: 0.0,
            stdev: 0.0,
            max: 0.0,
            stdev_percent: 0.0,
        }
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
            .map(|(avg, stdev, max, stdev_percent)| RequestSec {
                avg,
                stdev,
                max,
                stdev_percent,
            })
            .unwrap_or_default()
    }
}
