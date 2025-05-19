pub fn parse_to_milliseconds(value: &str) -> f64 {
    let value = value.trim();
    let (num_str, unit): (String, String) =
        value.chars().partition(|c| c.is_ascii_digit() || *c == '.');

    let num = num_str.parse::<f64>().unwrap_or(0.0);
    match unit.trim().to_lowercase().as_str() {
        "s" => num * 1000.0,
        "us" => num / 1000.0,
        // "ms" is the default
        _ => num,
    }
}

pub fn parse_count(value: &str) -> f64 {
    let value = value.trim();
    let (num_str, unit): (String, String) =
        value.chars().partition(|c| c.is_ascii_digit() || *c == '.');

    let num = num_str.parse::<f64>().unwrap_or(0.0);
    match unit.trim().to_lowercase().as_str() {
        "m" => num * 1_000_000.0,
        "k" => num * 1000.0,
        _ => num,
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

    #[test]
    fn test_parse_to_milliseconds() {
        assert_float_eq(parse_to_milliseconds("2s"), 2000.0);
        assert_float_eq(parse_to_milliseconds("500ms"), 500.0);
        assert_float_eq(parse_to_milliseconds("1000us"), 1.0);
        assert_float_eq(parse_to_milliseconds("42"), 42.0);
        assert_float_eq(parse_to_milliseconds("bad"), 0.0);
        assert_float_eq(parse_to_milliseconds("  3.5s  "), 3500.0);
        assert_float_eq(parse_to_milliseconds("10MS"), 10.0); // case insensitive
        assert_float_eq(parse_to_milliseconds("68.46%"), 68.46);
    }

    #[test]
    fn test_parse_count() {
        assert_float_eq(parse_count("56.20k"), 56200.0);
        assert_float_eq(parse_count("8.07k"), 8070.0);
        assert_float_eq(parse_count("62.00k"), 62000.0);
        assert_float_eq(parse_count("86.54%"), 86.54);
        assert_float_eq(parse_count("1.5M"), 1_500_000.0);
        assert_float_eq(parse_count("2.5"), 2.5);
    }
}
