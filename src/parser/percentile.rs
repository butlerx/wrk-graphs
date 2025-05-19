use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PercentileSpectrum {
    pub mean: f64,
    pub std_deviation: f64,
    pub max: f64,
    pub total_count: u64,
    pub buckets: u32,
    pub sub_buckets: u32,
    pub percentiles: Vec<PercentileBucket>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PercentileBucket {
    pub value: f64,
    pub percentile: f64,
    pub total_count: u64,
    pub inverse_percentile: f64,
}

impl Default for PercentileSpectrum {
    fn default() -> Self {
        PercentileSpectrum {
            mean: 0.0,
            std_deviation: 0.0,
            max: 0.0,
            total_count: 0,
            buckets: 0,
            sub_buckets: 0,
            percentiles: Vec::new(),
        }
    }
}

impl From<&[&str]> for PercentileSpectrum {
    fn from(lines: &[&str]) -> Self {
        lines
            .iter()
            .position(|l| l.contains("Detailed Percentile spectrum"))
            .map_or_else(PercentileSpectrum::default, |start_idx| {
                let summary = lines
                    .iter()
                    .skip(start_idx)
                    .take_while(|l| !l.starts_with("Latency"))
                    .fold(PercentileSpectrum::default(), |acc, line| match line {
                        l if l.starts_with("#[Mean") => {
                            let parts: Vec<_> = line.split_whitespace().collect();
                            let mean = match parts.get(2) {
                                Some(v) => v.trim_end_matches(',').parse().unwrap_or(0.0),
                                _ => 0.0,
                            };
                            let std_deviation = match parts.get(5) {
                                Some(v) => v.trim_end_matches(']').parse().unwrap_or(0.0),
                                _ => 0.0,
                            };
                            PercentileSpectrum {
                                mean,
                                std_deviation,
                                ..acc
                            }
                        }
                        l if l.starts_with("#[Max") => {
                            let parts: Vec<_> = line.split_whitespace().collect();
                            let max = match parts.get(2) {
                                Some(v) => v.trim_end_matches(',').parse().unwrap_or(0.0),
                                _ => 0.0,
                            };
                            let total_count = match parts.get(6) {
                                Some(v) => v.trim_end_matches(']').parse().unwrap_or(0),
                                _ => 0,
                            };
                            PercentileSpectrum {
                                max,
                                total_count,
                                ..acc
                            }
                        }
                        l if l.starts_with("#[Buckets") => {
                            let parts: Vec<_> = line.split_whitespace().collect();
                            let buckets = match parts.get(2) {
                                Some(v) => v.trim_end_matches(',').parse().unwrap_or(0),
                                _ => 0,
                            };
                            let sub_buckets = match parts.get(5) {
                                Some(v) => v.trim_end_matches(']').parse().unwrap_or(0),
                                _ => 0,
                            };
                            PercentileSpectrum {
                                buckets,
                                sub_buckets,
                                ..acc
                            }
                        }

                        _ => acc,
                    });

                let percentiles = lines
                    .iter()
                    .skip(start_idx + 2)
                    .take_while(|l| !l.starts_with("#["))
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        Some(PercentileBucket {
                            value: parts.first()?.parse().ok()?,
                            percentile: parts.get(1)?.parse().ok()?,
                            total_count: parts.get(2)?.parse().ok()?,
                            inverse_percentile: match parts.get(3) {
                                Some(&"inf") => f64::INFINITY,
                                Some(v) => v.parse().ok()?,
                                None => 0.0,
                            },
                        })
                    })
                    .collect();

                PercentileSpectrum {
                    percentiles,
                    ..summary
                }
            })
    }
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

    const SAMPLE_SPECTRUM: &str = r"
Detailed Percentile spectrum:
     Value   Percentile   TotalCount 1/(1-Percentile)

     0.921     0.000000            1         1.00
     4.053     0.100000         3951         1.11
     4.935     0.200000         7921         1.25
     5.627     0.300000        11858         1.43
     6.179     0.400000        15803         1.67
     6.671     0.500000        19783         inf
#[Mean    =        6.602, StdDeviation   =        1.919]
#[Max     =       12.496, Total count    =        39500]
#[Buckets =           27, SubBuckets     =         2048]
";

    #[test]
    fn test_parse_percentile_spectrum() {
        let lines: Vec<&str> = SAMPLE_SPECTRUM.lines().collect();
        let spectrum = PercentileSpectrum::from(lines.as_slice());

        assert_float_eq(spectrum.mean, 6.602);
        assert_float_eq(spectrum.std_deviation, 1.919);
        assert_float_eq(spectrum.max, 12.496);
        assert_eq!(spectrum.total_count, 39500);
        assert_eq!(spectrum.buckets, 27);
        assert_eq!(spectrum.sub_buckets, 2048);

        // Test first bucket
        assert_float_eq(spectrum.percentiles[0].value, 0.921);
        assert_float_eq(spectrum.percentiles[0].percentile, 0.0);
        assert_eq!(spectrum.percentiles[0].total_count, 1);
        assert_float_eq(spectrum.percentiles[0].inverse_percentile, 1.0);

        // Test middle bucket
        assert_float_eq(spectrum.percentiles[5].value, 6.671);
        assert_float_eq(spectrum.percentiles[5].percentile, 0.5);
        assert_eq!(spectrum.percentiles[5].total_count, 19783);
        assert_eq!(spectrum.percentiles[5].inverse_percentile, f64::INFINITY);
    }
}
