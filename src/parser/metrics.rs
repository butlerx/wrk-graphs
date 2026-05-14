use super::{
    is_empty, latency::Latency, percentile::PercentileBucket, request_sec::RequestSec, units,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct WrkMetrics {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub endpoint: String,
    #[serde(default, skip_serializing_if = "is_empty::check_u64")]
    pub threads: u64,
    #[serde(default, skip_serializing_if = "is_empty::check_u64")]
    pub connections: u64,
    #[serde(default, skip_serializing_if = "Latency::is_empty")]
    pub latency: Latency,
    #[serde(default, skip_serializing_if = "RequestSec::is_empty")]
    pub req: RequestSec,
    #[serde(default, skip_serializing_if = "is_empty::check_u64")]
    pub total_requests: u64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub duration: f64,
    #[serde(default, skip_serializing_if = "is_empty::check_f64")]
    pub requests_per_sec: f64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transfer_per_sec: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub latency_distribution: HashMap<String, f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub percentiles: Vec<PercentileBucket>,
}

impl From<&str> for WrkMetrics {
    fn from(output: &str) -> Self {
        let lines = output
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();

        let endpoint = lines
            .iter()
            .find(|l| l.starts_with("Running"))
            .and_then(|l| l.split('@').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let (threads, connections) = lines
            .iter()
            .find(|l| l.contains("threads and"))
            .and_then(|l| {
                let mut parts = l.split_whitespace();
                let threads = parts.next()?.parse().ok()?;
                let connections = parts.nth(2)?.parse().ok()?;
                Some((threads, connections))
            })
            .unwrap_or((0, 0));

        let latency = lines
            .iter()
            .find(|l| l.starts_with("Latency"))
            .map(|&l| Latency::from(l))
            .unwrap_or_default();

        let latency_distribution = parse_latency_distribution(&lines);
        let percentiles = lines
            .iter()
            .position(|l| l.contains("Detailed Percentile spectrum"))
            .map_or_else(Vec::new, |start_idx| {
                lines
                    .iter()
                    .skip(start_idx + 2)
                    .take_while(|l| !l.starts_with("#["))
                    .filter_map(|l| {
                        if l.is_empty() {
                            None
                        } else {
                            PercentileBucket::try_from(*l).ok()
                        }
                    })
                    .collect()
            });

        let req = lines
            .iter()
            .find(|l| l.starts_with("Req/Sec"))
            .map(|&l| RequestSec::from(l))
            .unwrap_or_default();

        let (total_requests, duration) = lines
            .iter()
            .find(|l| l.contains("requests in"))
            .and_then(|l| parse_requests_line(l))
            .unwrap_or((0, 0.0));

        let requests_per_sec = lines
            .iter()
            .find(|l| l.contains("Requests/sec:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| f64::from_str(s).ok())
            .unwrap_or(0.0);

        let transfer_per_sec = lines
            .iter()
            .find(|l| l.contains("Transfer/sec:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .map(String::from)
            .unwrap_or_default();

        WrkMetrics {
            endpoint,
            threads,
            connections,
            latency,
            req,
            total_requests,
            duration,
            requests_per_sec,
            transfer_per_sec,
            latency_distribution,
            percentiles,
        }
    }
}

fn parse_latency_distribution(lines: &[&str]) -> HashMap<String, f64> {
    match lines
        .iter()
        .position(|l| l.contains("Latency Distribution"))
    {
        Some(idx) => lines
            .iter()
            .skip(idx + 1)
            .take_while(|l| !l.is_empty())
            .filter(|l| l.contains('%'))
            .filter_map(|l| {
                let parts = l.split_whitespace().collect::<Vec<_>>();
                if parts.len() >= 2 {
                    let percent = parts[0].to_string();
                    let value = units::parse_to_milliseconds(parts[1]);
                    Some((percent, value))
                } else {
                    None
                }
            })
            .collect::<HashMap<String, f64>>(),
        None => HashMap::new(),
    }
}

/// Parses the line containing the total requests and duration
/// Returns a tuple of total requests and duration in seconds
fn parse_requests_line(line: &str) -> Option<(u64, f64)> {
    let mut parts = line.split_whitespace();
    let requests = parts.next()?.parse().ok()?;

    let (duration_str, unit): (String, String) = parts
        .nth(2)?
        .trim_end_matches(',')
        .chars()
        .partition(|c| c.is_ascii_digit() || *c == '.');

    if unit.is_empty() {
        // if no unit is provided, assume seconds
        let duration = duration_str.parse().ok()?;
        return Some((requests, duration));
    }

    let duration_num = duration_str.parse::<f64>().ok()?;
    let duration = match unit.trim().to_lowercase().as_str() {
        "h" => duration_num * 3600.0,
        "m" => duration_num * 60.0,
        _ => duration_num,
    };

    Some((requests, duration))
}

pub fn parse_tests(output: &str) -> Vec<WrkMetrics> {
    let mut tests = Vec::new();
    let mut current_test = String::new();

    for line in output.lines() {
        if line.trim().starts_with("Running") && !current_test.trim().is_empty() {
            tests.push(WrkMetrics::from(current_test.as_str()));
            current_test.clear();
        }
        current_test.push_str(line);
        current_test.push('\n');
    }

    // Parse the last test
    if current_test.trim().starts_with("Running") && !current_test.trim().is_empty() {
        tests.push(WrkMetrics::from(current_test.as_str()));
    }

    tests
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_float_eq(a: f64, b: f64) {
        const EPSILON: f64 = 1e-6;
        assert!(
            (a - b).abs() < EPSILON,
            "Expected {a} to be approximately equal to {b}"
        );
    }

    const SAMPLE_OUTPUT: &str = include_str!("fixtures/wrk1_basic.txt");

    #[test]
    fn test_wrk_metrics_from() {
        let metrics = WrkMetrics::from(SAMPLE_OUTPUT);
        assert_eq!(metrics.endpoint, "http://localhost:8080");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 125.12);
        assert_float_eq(metrics.latency.stddev, 25.31);
        assert_float_eq(metrics.latency.max, 450.0);
        assert_float_eq(metrics.req.avg, 400.12);
        assert_float_eq(metrics.req.stddev, 50.23);
        assert_float_eq(metrics.req.max, 550.0);
        assert_eq!(metrics.total_requests, 8000);
        assert_float_eq(metrics.duration, 10.0);
        assert_float_eq(metrics.requests_per_sec, 800.12);
        assert_eq!(metrics.transfer_per_sec, "656.56KB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50%"], 120.12);
        assert_float_eq(dist["75%"], 130.0);
        assert_float_eq(dist["90%"], 140.23);
        assert_float_eq(dist["99%"], 400.0);
    }

    const SAMPLE_OUTPUT_2: &str = include_str!("fixtures/wrk1_us_units.txt");

    #[test]
    fn test_wrk_metrics_from_2() {
        let metrics = WrkMetrics::from(SAMPLE_OUTPUT_2);
        assert_eq!(metrics.endpoint, "http://localhost:8080/index.html");
        assert_eq!(metrics.threads, 12);
        assert_eq!(metrics.connections, 400);
        assert_float_eq(metrics.latency.avg, 0.63591);
        assert_float_eq(metrics.latency.stddev, 0.89);
        assert_float_eq(metrics.latency.max, 12.92);
        assert_float_eq(metrics.req.avg, 56200.0);
        assert_float_eq(metrics.req.stddev, 8070.0);
        assert_float_eq(metrics.req.max, 62000.0);
        assert_eq!(metrics.total_requests, 22_464_657);
        assert_float_eq(metrics.duration, 30.0);
        assert_float_eq(metrics.requests_per_sec, 748_868.53);
        assert_eq!(metrics.transfer_per_sec, "606.33MB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50%"], 0.25);
        assert_float_eq(dist["75%"], 0.491);
        assert_float_eq(dist["90%"], 0.7);
        assert_float_eq(dist["99%"], 5.8);
    }

    const WRK2_INPUT: &str = include_str!("fixtures/wrk2_full.txt");

    #[test]
    fn test_parse_wrk2_output() {
        let metrics = WrkMetrics::from(WRK2_INPUT);
        assert_eq!(metrics.endpoint, "http://127.0.0.1:8080/sys/ping");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 1.46);
        assert_float_eq(metrics.latency.stddev, 2.24);
        assert_float_eq(metrics.latency.max, 44.06);
        assert_float_eq(metrics.req.avg, 1050.0);
        assert_float_eq(metrics.req.stddev, 265.56);
        assert_float_eq(metrics.req.max, 5400.0);
        assert_eq!(metrics.total_requests, 119_802);
        assert_float_eq(metrics.duration, 60.0);
        assert_float_eq(metrics.requests_per_sec, 1996.65);
        assert_eq!(metrics.transfer_per_sec, "376.32KB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50.000%"], 1.18);
        assert_float_eq(dist["75.000%"], 1.54);
        assert_float_eq(dist["90.000%"], 1.93);
        assert_float_eq(dist["99.000%"], 11.72);
        assert_float_eq(dist["99.900%"], 30.74);
        assert_float_eq(dist["99.990%"], 39.52);
        assert_float_eq(dist["99.999%"], 44.03);
        assert_float_eq(dist["100.000%"], 44.10);
    }

    const WRK2_INPUT_2: &str = include_str!("fixtures/wrk2_short.txt");

    #[test]
    fn test_parse_wrk2_output_2() {
        let metrics = WrkMetrics::from(WRK2_INPUT_2);
        assert_eq!(metrics.endpoint, "http://127.0.0.1:80/index.html");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 6.60);
        assert_float_eq(metrics.latency.stddev, 1.92);
        assert_float_eq(metrics.latency.max, 12.50);
        assert_float_eq(metrics.req.avg, 1040.0);
        assert_float_eq(metrics.req.stddev, 1080.0);
        assert_float_eq(metrics.req.max, 2500.0);
        assert_eq!(metrics.total_requests, 60018);
        assert_float_eq(metrics.duration, 30.0);
        assert_float_eq(metrics.requests_per_sec, 2000.28);
        assert_eq!(metrics.transfer_per_sec, "676.18KB");

        // Test latency distribution
        let dist = &metrics.latency_distribution;
        assert_float_eq(dist["50.000%"], 6.67);
        assert_float_eq(dist["75.000%"], 7.78);
        assert_float_eq(dist["90.000%"], 9.14);
        assert_float_eq(dist["99.000%"], 11.18);
        assert_float_eq(dist["99.900%"], 12.30);
        assert_float_eq(dist["99.990%"], 12.45);
        assert_float_eq(dist["99.999%"], 12.50);
        assert_float_eq(dist["100.000%"], 12.50);
    }

    const WRK_CALIBRATION_NO_HISTOGRAM: &str =
        include_str!("fixtures/wrk_calibration_no_histogram.txt");

    #[test]
    fn test_parse_wrk_calibration_no_histogram() {
        let metrics = WrkMetrics::from(WRK_CALIBRATION_NO_HISTOGRAM);
        assert_eq!(metrics.endpoint, "http://127.0.0.1:80/index.html");
        assert_eq!(metrics.threads, 2);
        assert_eq!(metrics.connections, 100);
        assert_float_eq(metrics.latency.avg, 6.46);
        assert_float_eq(metrics.latency.stddev, 1.93);
        assert_float_eq(metrics.latency.max, 12.34);
        assert_float_eq(metrics.req.avg, 1050.0);
        assert_float_eq(metrics.req.stddev, 1120.0);
        assert_float_eq(metrics.req.max, 2500.0);
        assert_eq!(metrics.total_requests, 60017);
        assert_float_eq(metrics.duration, 30.01);
        assert_float_eq(metrics.requests_per_sec, 2000.15);
        assert_eq!(metrics.transfer_per_sec, "676.14KB");

        // No latency distribution or percentile data
        assert!(metrics.latency_distribution.is_empty());
        assert!(metrics.percentiles.is_empty());
    }

    #[test]
    fn test_error_handling() {
        let empty = WrkMetrics::from("invalid output");
        assert_eq!(empty.endpoint, "");
        assert_eq!(empty.threads, 0);
        assert_eq!(empty.connections, 0);
        assert_float_eq(empty.latency.avg, 0.0);
        assert_float_eq(empty.latency.stddev, 0.0);
        assert_float_eq(empty.latency.max, 0.0);
        assert_float_eq(empty.req.avg, 0.0);
        assert_float_eq(empty.req.stddev, 0.0);
        assert_float_eq(empty.req.max, 0.0);
        assert_eq!(empty.total_requests, 0);
        assert_float_eq(empty.duration, 0.0);
        assert_float_eq(empty.requests_per_sec, 0.0);
        assert_eq!(empty.transfer_per_sec, "");
        assert!(empty.latency_distribution.is_empty());
        assert_eq!(empty.percentiles.len(), 0);
    }

    #[test]
    fn test_parse_multiple_tests() {
        let input = include_str!("fixtures/wrk_multiple_tests.txt");

        let collection = parse_tests(input);
        assert_eq!(collection.len(), 2);

        // Test first result
        let first = &collection[0];
        assert_eq!(first.endpoint, "http://google.fr");
        assert_eq!(first.threads, 2);
        assert_eq!(first.connections, 10);
        assert_float_eq(first.latency.avg, 30.56);
        assert_float_eq(first.latency.stddev, 12.92);
        assert_float_eq(first.latency.max, 140.67);
        assert_eq!(first.total_requests, 1495);
        assert_float_eq(first.duration, 5.0);
        assert_float_eq(first.requests_per_sec, 1.0);
        assert_eq!(first.transfer_per_sec, "156.95KB");

        // Test second result
        let second = &collection[1];
        assert_eq!(second.endpoint, "http://google.fr");
        assert_eq!(second.threads, 2);
        assert_eq!(second.connections, 10);
        assert_float_eq(second.latency.avg, 30.56);
        assert_float_eq(second.latency.stddev, 12.92);
        assert_float_eq(second.latency.max, 140.67);
        assert_eq!(second.total_requests, 1495);
        assert_float_eq(second.duration, 5.0);
        assert_float_eq(second.requests_per_sec, 2.0);
        assert_eq!(second.transfer_per_sec, "156.95KB");
    }

    #[test]
    fn test_parse_empty_input() {
        let collection = parse_tests("");
        assert!(collection.is_empty());
    }

    #[test]
    fn test_parse_invalid_input() {
        let collection = parse_tests("invalid output");
        assert!(collection.is_empty());
    }
}
