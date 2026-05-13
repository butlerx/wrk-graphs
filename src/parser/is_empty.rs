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
