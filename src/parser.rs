pub mod criterion;
mod is_empty;
mod latency;
mod metrics;
mod percentile;
mod request_sec;
mod units;

pub use criterion::CriterionMetrics;
pub use metrics::{parse_tests, WrkMetrics};
pub use percentile::PercentileBucket;

use serde::{Deserialize, Serialize};

/// Unified result type for benchmark/loadtest data from any supported tool.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BenchmarkResult {
    Wrk(Box<WrkMetrics>),
    Criterion(Box<CriterionMetrics>),
}

/// Auto-detect input format and parse into unified results.
/// Tries criterion formats first (JSON, sample.json, CLI), then falls back to wrk.
pub fn parse_input(output: &str) -> Vec<BenchmarkResult> {
    if let Some(criterion_results) = criterion::try_parse(output) {
        return criterion_results
            .into_iter()
            .map(|m| BenchmarkResult::Criterion(Box::new(m)))
            .collect();
    }

    let wrk_results = parse_tests(output);
    if !wrk_results.is_empty() {
        return wrk_results
            .into_iter()
            .map(|m| BenchmarkResult::Wrk(Box::new(m)))
            .collect();
    }

    Vec::new()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_input_detects_wrk() {
        let input = include_str!("parser/fixtures/wrk1_basic.txt");
        let results = parse_input(input);
        assert!(!results.is_empty());
        assert!(matches!(results[0], BenchmarkResult::Wrk(_)));
    }

    #[test]
    fn parse_input_detects_criterion_cli() {
        let input = include_str!("parser/fixtures/criterion_cli_simple.txt");
        let results = parse_input(input);
        assert!(!results.is_empty());
        assert!(matches!(results[0], BenchmarkResult::Criterion(_)));
    }

    #[test]
    fn parse_input_detects_criterion_json() {
        let input = include_str!("parser/fixtures/criterion_json_output.json");
        let results = parse_input(input);
        assert!(!results.is_empty());
        assert!(matches!(results[0], BenchmarkResult::Criterion(_)));
    }

    #[test]
    fn parse_input_detects_criterion_sample_json() {
        let input = include_str!("parser/fixtures/criterion_sample.json");
        let results = parse_input(input);
        assert!(!results.is_empty());
        assert!(matches!(results[0], BenchmarkResult::Criterion(_)));
    }

    #[test]
    fn parse_input_empty_returns_empty() {
        let results = parse_input("");
        assert!(results.is_empty());
    }

    #[test]
    fn parse_input_garbage_returns_empty() {
        let results = parse_input("this is not benchmark output at all");
        assert!(results.is_empty());
    }

    #[test]
    fn parse_input_criterion_takes_priority_over_wrk() {
        let input = include_str!("parser/fixtures/criterion_cli_simple.txt");
        let results = parse_input(input);
        assert!(
            matches!(results[0], BenchmarkResult::Criterion(_)),
            "Criterion should be detected before wrk fallback"
        );
    }

    #[test]
    fn parse_input_wrk2_detected() {
        let input = include_str!("parser/fixtures/wrk2_full.txt");
        let results = parse_input(input);
        assert!(!results.is_empty());
        assert!(matches!(results[0], BenchmarkResult::Wrk(_)));
    }
}
