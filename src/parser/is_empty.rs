// serde's `skip_serializing_if` requires `fn(&T) -> bool` signature
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn check_u64(value: &u64) -> bool {
    *value == 0
}

// serde's `skip_serializing_if` requires `fn(&T) -> bool` signature
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn check_f64(value: &f64) -> bool {
    *value == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_u64_zero_is_empty() {
        assert!(check_u64(&0));
    }

    #[test]
    fn check_u64_nonzero_is_not_empty() {
        assert!(!check_u64(&1));
        assert!(!check_u64(&u64::MAX));
    }

    #[test]
    fn check_f64_zero_is_empty() {
        assert!(check_f64(&0.0));
    }

    #[test]
    fn check_f64_nonzero_is_not_empty() {
        assert!(!check_f64(&1.0));
        assert!(!check_f64(&-1.0));
        assert!(!check_f64(&f64::MIN_POSITIVE));
    }
}
