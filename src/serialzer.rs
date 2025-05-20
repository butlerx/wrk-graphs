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
    #[serde(default, skip_serializing_if = "parser::WrkMetrics::is_empty")]
    pub metrics: parser::WrkMetrics,
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
    let metrics = parser::WrkMetrics::from(data);
    let description = if desc.is_empty() { None } else { Some(desc) };
    let data_obj = Loadtest {
        metrics,
        description,
        tags,
    };

    let mut buf = Vec::new();
    data_obj
        .serialize(&mut rmp_serde::Serializer::new(&mut buf))
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
        assert_eq!(decoded.metrics.endpoint, "http://localhost:8080");
        assert_eq!(decoded.description, Some(description));
        assert_eq!(decoded.tags, tags);
    }

    #[test]
    fn test_invalid_hash() {
        let invalid_hash = "invalid_base64";
        let decoded = decode_dashboard(invalid_hash);
        assert!(decoded.is_err());
    }
}
