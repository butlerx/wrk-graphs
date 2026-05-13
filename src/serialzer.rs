use crate::parser;
use base64::prelude::*;
use flate2::{read::ZlibDecoder, read::ZlibEncoder};
use serde::{Deserialize, Serialize};
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to decode base64: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Failed to deserialize data: {0}")]
    Deserialize(#[from] rmp_serde::decode::Error),
    #[error("Failed to compress/decompress data: {0}")]
    Compression(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Loadtest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<parser::WrkMetrics>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub benchmarks: Vec<parser::CriterionMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

pub fn decode_dashboard(hash: &str) -> Result<Loadtest, Error> {
    let data = BASE64_URL_SAFE_NO_PAD.decode(hash)?;
    let mut decoder = ZlibDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    let loadtest = rmp_serde::from_slice::<Loadtest>(&decompressed)?;
    Ok(loadtest)
}

pub fn encode_dashboard(data: &str, desc: String, tags: Vec<String>) -> String {
    let results = parser::parse_input(data);

    let mut tests = Vec::new();
    let mut benchmarks = Vec::new();

    for result in results {
        match result {
            parser::BenchmarkResult::Wrk(m) => tests.push(*m),
            parser::BenchmarkResult::Criterion(m) => benchmarks.push(*m),
        }
    }

    let description = if desc.is_empty() { None } else { Some(desc) };
    let data_obj = Loadtest {
        tests,
        benchmarks,
        description,
        tags,
    };

    let mut buf = Vec::new();
    data_obj
        .serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
        .unwrap();

    let mut encoder = ZlibEncoder::new(&buf[..], flate2::Compression::best());
    let mut compressed = Vec::new();
    encoder.read_to_end(&mut compressed).unwrap();
    BASE64_URL_SAFE_NO_PAD.encode(compressed)
}

#[cfg(test)]
mod test {
    use super::*;

    const SAMPLE_INPUT: &str = r"
Running 10s test @ http://localhost:8080
  2 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   125.12ms   25.31ms 450.00ms   90.12%
    Req/Sec   400.12     50.23   550.00     85.45%
  Latency Distribution
     50%  120.12ms
     75%  130.00ms
     90%  140.23ms
     99%  400.00ms
  8000 requests in 10.00s, 6.42MB read
Requests/sec:    800.12
Transfer/sec:    656.56KB
";

    #[test]
    fn test_encode_decode() {
        let description = "Test description".to_string();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let hash = encode_dashboard(SAMPLE_INPUT, description.clone(), tags.clone());
        let decoded = decode_dashboard(&hash).unwrap();
        assert_eq!(decoded.tests[0].endpoint, "http://localhost:8080");
        assert!(decoded.benchmarks.is_empty());
        assert_eq!(decoded.description, Some(description));
        assert_eq!(decoded.tags, tags);
    }

    #[test]
    fn test_encode_decode_criterion() {
        let criterion_input = r"
Benchmarking fib/20
fib/20                  time:   [1.9245 ms 1.9298 ms 1.9359 ms]
                        change: [-0.5765% +0.2437% +1.1291%] (p = 0.59 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
";
        let hash = encode_dashboard(criterion_input, String::new(), vec![]);
        let decoded = decode_dashboard(&hash).unwrap();
        assert!(decoded.tests.is_empty());
        assert_eq!(decoded.benchmarks.len(), 1);
        assert_eq!(decoded.benchmarks[0].name, "fib/20");
    }

    #[test]
    fn test_invalid_hash() {
        let invalid_hash = "invalid_base64";
        let decoded = decode_dashboard(invalid_hash);
        assert!(decoded.is_err());
    }
}
