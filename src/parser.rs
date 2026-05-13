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
