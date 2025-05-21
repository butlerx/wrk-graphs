use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PercentileBucket {
    pub value: f64,
    pub percentile: f64,
}

impl TryFrom<&str> for PercentileBucket {
    type Error = String;
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let value = parts
            .first()
            .ok_or("Invalid line format")?
            .parse::<f64>()
            .map_err(|_| "Invalid value".to_string())?;
        let percentile = parts
            .get(1)
            .ok_or("Invalid line format")?
            .parse::<f64>()
            .map_err(|_| "Invalid percentile".to_string())?;
        Ok(Self { value, percentile })
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

        // Test first bucket
        assert_float_eq(spectrum.percentiles[0].value, 0.921);
        assert_float_eq(spectrum.percentiles[0].percentile, 0.0);

        // Test middle bucket
        assert_float_eq(spectrum.percentiles[5].value, 6.671);
        assert_float_eq(spectrum.percentiles[5].percentile, 50.0);
    }
}
