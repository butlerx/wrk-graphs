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

    const SAMPLE_SPECTRUM: &str = include_str!("fixtures/percentile_spectrum.txt");

    #[test]
    fn test_parse_percentile_spectrum() {
        let percentiles = SAMPLE_SPECTRUM
            .lines()
            .filter_map(|l| PercentileBucket::try_from(l).ok())
            .collect::<Vec<PercentileBucket>>();

        // Test first bucket
        assert_float_eq(percentiles[0].value, 0.921);
        assert_float_eq(percentiles[0].percentile, 0.0);

        // Test middle bucket
        assert_float_eq(percentiles[5].value, 6.671);
        assert_float_eq(percentiles[5].percentile, 0.5);
    }
}
