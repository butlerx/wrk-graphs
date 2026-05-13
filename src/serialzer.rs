use crate::parser::{self, CriterionMetrics, PercentileBucket};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Read;
use thiserror::Error;

/// Version byte prepended to brotli-compressed payloads to distinguish from legacy zlib.
/// Zlib streams never start with 0x00 (CMF byte always has CM=8 in low nibble).
const BROTLI_VERSION_BYTE: u8 = 0x00;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to decode base64: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Failed to deserialize data: {0}")]
    Deserialize(#[from] rmp_serde::decode::Error),
    #[error("Failed to compress/decompress data: {0}")]
    Compression(#[from] std::io::Error),
    #[error("Encoded URL is too long to share ({length} chars, max {max})")]
    UrlTooLong { length: usize, max: usize },
}

/// Maximum allowed length for the base64-encoded hash fragment.
/// Keeps full URLs under ~8 KB which is safe for sharing via messaging
/// platforms, email clients, and browser navigation.
const MAX_HASH_LENGTH: usize = 8_000;

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

    let decompressed = if data.first() == Some(&BROTLI_VERSION_BYTE) {
        let mut output = Vec::new();
        brotli::BrotliDecompress(&mut &data[1..], &mut output)?;
        output
    } else {
        let mut output = Vec::new();
        flate2::read::ZlibDecoder::new(&data[..]).read_to_end(&mut output)?;
        output
    };

    let loadtest = rmp_serde::from_slice::<Loadtest>(&decompressed)?;
    Ok(loadtest)
}

/// Maximum number of percentile buckets to keep when encoding for URL sharing.
/// Points are selected with logarithmic spacing to preserve tail detail.
const MAX_PERCENTILE_BUCKETS: usize = 25;

/// Maximum number of criterion sample points to keep when encoding for URL sharing.
const MAX_CRITERION_SAMPLES: usize = 50;

/// Downsample percentile buckets using logarithmic spacing.
///
/// Percentile data is log-distributed (most interesting detail is in the tail:
/// p99, p99.9, p99.99). Logarithmic index selection preserves that tail detail
/// while thinning out the dense lower percentiles.
fn downsample_percentiles(buckets: &[PercentileBucket]) -> Vec<PercentileBucket> {
    if buckets.len() <= MAX_PERCENTILE_BUCKETS {
        return buckets.to_vec();
    }

    let n = buckets.len();
    let target = MAX_PERCENTILE_BUCKETS;

    let mut indices: Vec<usize> = Vec::with_capacity(target);
    indices.push(0);

    #[allow(clippy::cast_precision_loss)]
    let log_max = ((n - 1) as f64).ln_1p();
    for i in 1..target - 1 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let idx = (((i as f64 / (target - 1) as f64) * log_max)
            .exp_m1()
            .round()) as usize;
        let idx = idx.min(n - 1);
        if indices.last() != Some(&idx) {
            indices.push(idx);
        }
    }

    if indices.last() != Some(&(n - 1)) {
        indices.push(n - 1);
    }

    indices.iter().map(|&i| buckets[i].clone()).collect()
}

fn downsample_samples(iteration_count: &[f64], measured_values: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = iteration_count.len();
    if n <= MAX_CRITERION_SAMPLES {
        return (iteration_count.to_vec(), measured_values.to_vec());
    }

    let indices: Vec<usize> = (0..MAX_CRITERION_SAMPLES)
        .map(|i| i * (n - 1) / (MAX_CRITERION_SAMPLES - 1))
        .collect();

    let iters = indices.iter().map(|&i| iteration_count[i]).collect();
    let values = indices.iter().map(|&i| measured_values[i]).collect();
    (iters, values)
}

fn compact_criterion(mut m: CriterionMetrics) -> CriterionMetrics {
    if m.iteration_count.len() > MAX_CRITERION_SAMPLES {
        let (iters, values) = downsample_samples(&m.iteration_count, &m.measured_values);
        m.iteration_count = iters;
        m.measured_values = values;
    }
    if let Some(ref mut baseline) = m.baseline {
        if baseline.iteration_count.len() > MAX_CRITERION_SAMPLES {
            let (iters, values) =
                downsample_samples(&baseline.iteration_count, &baseline.measured_values);
            baseline.iteration_count = iters;
            baseline.measured_values = values;
        }
    }
    m
}

pub fn encode_dashboard(data: &str, desc: String, tags: Vec<String>) -> Result<String, Error> {
    let results = parser::parse_input(data);

    let mut tests = Vec::new();
    let mut benchmarks = Vec::new();

    for result in results {
        match result {
            parser::BenchmarkResult::Wrk(mut m) => {
                m.percentiles = downsample_percentiles(&m.percentiles);
                tests.push(*m);
            }
            parser::BenchmarkResult::Criterion(m) => benchmarks.push(compact_criterion(*m)),
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

    let mut compressed = Vec::new();
    compressed.push(BROTLI_VERSION_BYTE);

    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        ..Default::default()
    };
    brotli::BrotliCompress(&mut &buf[..], &mut compressed, &params)?;

    let hash = BASE64_URL_SAFE_NO_PAD.encode(compressed);

    if hash.len() > MAX_HASH_LENGTH {
        return Err(Error::UrlTooLong {
            length: hash.len(),
            max: MAX_HASH_LENGTH,
        });
    }

    Ok(hash)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::PercentileBucket;

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
        let hash = encode_dashboard(SAMPLE_INPUT, description.clone(), tags.clone()).unwrap();
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
        let hash = encode_dashboard(criterion_input, String::new(), vec![]).unwrap();
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

    #[test]
    fn test_decode_legacy_zlib_format() {
        use flate2::read::ZlibEncoder;

        let data_obj = Loadtest {
            tests: vec![],
            benchmarks: vec![],
            description: Some("legacy".to_string()),
            tags: vec![],
        };
        let mut buf = Vec::new();
        data_obj
            .serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
            .unwrap();

        let mut encoder = ZlibEncoder::new(&buf[..], flate2::Compression::best());
        let mut compressed = Vec::new();
        encoder.read_to_end(&mut compressed).unwrap();
        let hash = BASE64_URL_SAFE_NO_PAD.encode(&compressed);

        let decoded = decode_dashboard(&hash).unwrap();
        assert_eq!(decoded.description, Some("legacy".to_string()));
    }

    #[test]
    fn test_brotli_encoding_has_version_byte() {
        let hash = encode_dashboard(SAMPLE_INPUT, String::new(), vec![]).unwrap();
        let raw = BASE64_URL_SAFE_NO_PAD.decode(&hash).unwrap();
        assert_eq!(raw[0], BROTLI_VERSION_BYTE);
    }

    #[test]
    fn test_downsample_percentiles_passthrough_when_small() {
        let buckets: Vec<PercentileBucket> = (0..10)
            .map(|i| PercentileBucket {
                value: i as f64,
                percentile: i as f64 / 10.0,
            })
            .collect();
        let result = downsample_percentiles(&buckets);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_downsample_percentiles_reduces_large_input() {
        let buckets: Vec<PercentileBucket> = (0..100)
            .map(|i| PercentileBucket {
                value: i as f64,
                percentile: i as f64 / 100.0,
            })
            .collect();
        let result = downsample_percentiles(&buckets);
        assert!(result.len() <= MAX_PERCENTILE_BUCKETS);
        assert_eq!(result.first().unwrap().value, 0.0);
        assert_eq!(result.last().unwrap().value, 99.0);
    }

    #[test]
    fn test_downsample_samples_passthrough_when_small() {
        let iters: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let values: Vec<f64> = (0..30).map(|i| i as f64 * 100.0).collect();
        let (r_iters, r_values) = downsample_samples(&iters, &values);
        assert_eq!(r_iters.len(), 30);
        assert_eq!(r_values.len(), 30);
    }

    #[test]
    fn test_downsample_samples_reduces_large_input() {
        let iters: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let values: Vec<f64> = (0..200).map(|i| i as f64 * 100.0).collect();
        let (r_iters, r_values) = downsample_samples(&iters, &values);
        assert_eq!(r_iters.len(), MAX_CRITERION_SAMPLES);
        assert_eq!(r_values.len(), MAX_CRITERION_SAMPLES);
        assert_eq!(r_iters[0], 0.0);
        assert_eq!(*r_iters.last().unwrap(), 199.0);
    }

    #[test]
    fn test_rejects_url_too_long() {
        // Use a pseudo-random description that resists brotli compression.
        let big_desc: String = (0u64..2000)
            .map(|i| format!("{:08x}", i.wrapping_mul(2_654_435_761)))
            .collect();
        let result = encode_dashboard(SAMPLE_INPUT, big_desc, vec![]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::UrlTooLong { .. }),
            "Expected UrlTooLong, got: {err:?}"
        );
    }
}
