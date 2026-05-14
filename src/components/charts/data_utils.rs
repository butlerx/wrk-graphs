#![allow(clippy::cast_precision_loss)]

/// Compute per-iteration time in milliseconds from raw Criterion sample data.
///
/// Each Criterion sample records `iteration_count` (number of iterations in the sample)
/// and `measured_values` (total measured nanoseconds for that sample). This divides to get
/// the average time per iteration in milliseconds.
pub fn compute_per_iteration_ms(iteration_count: &[f64], measured_values: &[f64]) -> Vec<f64> {
    iteration_count
        .iter()
        .zip(measured_values.iter())
        .filter_map(|(iters, measured_ns)| {
            if *iters > 0.0 && iters.is_finite() && measured_ns.is_finite() {
                Some((measured_ns / iters) / 1_000_000.0)
            } else {
                None
            }
        })
        .collect()
}

/// Compute a Gaussian Kernel Density Estimate for the given values.
///
/// Returns `points` evenly-spaced (x, density) pairs spanning the data range.
/// Uses Silverman's rule of thumb for bandwidth selection.
pub fn compute_kde(values: &[f64], points: usize) -> Vec<(f64, f64)> {
    if values.is_empty() || points < 2 {
        return Vec::new();
    }

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt();

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }

    let x_min = if (max - min).abs() < f64::EPSILON {
        min - 1.0
    } else {
        min
    };
    let x_max = if (max - min).abs() < f64::EPSILON {
        max + 1.0
    } else {
        max
    };

    let silverman = 1.06 * sigma * n.powf(-0.2);
    let h = if silverman.is_finite() && silverman > 0.0 {
        silverman
    } else {
        ((x_max - x_min).abs() / 20.0).max(1e-9)
    };

    let step = (x_max - x_min) / (points as f64 - 1.0);
    let norm = 1.0 / ((2.0 * std::f64::consts::PI).sqrt() * h * n);

    (0..points)
        .map(|i| {
            let x = x_min + i as f64 * step;
            let sum = values
                .iter()
                .map(|v| {
                    let u = (x - v) / h;
                    (-0.5 * u * u).exp()
                })
                .sum::<f64>();
            (x, norm * sum)
        })
        .collect()
}

/// Compute scatter-plot points from raw Criterion sample data.
///
/// When `has_slope` is true (regression data available), returns `(iterations, total_time_ms)`.
/// Otherwise, returns `(sample_index, avg_iteration_time_ms)`.
pub fn compute_regression_points(
    iteration_count: &[f64],
    measured_values: &[f64],
    has_slope: bool,
) -> Vec<(f64, f64)> {
    if has_slope {
        iteration_count
            .iter()
            .zip(measured_values.iter())
            .filter_map(|(iters, measured_ns)| {
                if iters.is_finite() && measured_ns.is_finite() {
                    Some((*iters, measured_ns / 1_000_000.0))
                } else {
                    None
                }
            })
            .collect()
    } else {
        iteration_count
            .iter()
            .zip(measured_values.iter())
            .enumerate()
            .filter_map(|(idx, (iters, measured_ns))| {
                if *iters > 0.0 && iters.is_finite() && measured_ns.is_finite() {
                    Some(((idx + 1) as f64, (measured_ns / iters) / 1_000_000.0))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // --- compute_per_iteration_ms ---

    #[test]
    fn per_iteration_ms_basic() {
        let iters = vec![100.0, 200.0];
        let measured = vec![500_000_000.0, 800_000_000.0]; // ns
        let result = compute_per_iteration_ms(&iters, &measured);
        assert_eq!(result.len(), 2);
        assert!((result[0] - 5.0).abs() < 1e-9); // 500M ns / 100 iters / 1M = 5 ms
        assert!((result[1] - 4.0).abs() < 1e-9); // 800M ns / 200 iters / 1M = 4 ms
    }

    #[test]
    fn per_iteration_ms_filters_zero_iters() {
        let iters = vec![0.0, 100.0];
        let measured = vec![500_000_000.0, 200_000_000.0];
        let result = compute_per_iteration_ms(&iters, &measured);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn per_iteration_ms_filters_nan() {
        let iters = vec![f64::NAN, 100.0];
        let measured = vec![500_000_000.0, 100_000_000.0];
        let result = compute_per_iteration_ms(&iters, &measured);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn per_iteration_ms_filters_infinity() {
        let iters = vec![f64::INFINITY, 50.0];
        let measured = vec![100_000_000.0, 50_000_000.0];
        let result = compute_per_iteration_ms(&iters, &measured);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn per_iteration_ms_empty_input() {
        let result = compute_per_iteration_ms(&[], &[]);
        assert!(result.is_empty());
    }

    // --- compute_kde ---

    #[test]
    fn kde_basic_shape() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = compute_kde(&values, 10);
        assert_eq!(result.len(), 10);
        // First x should be near min, last near max
        assert!((result[0].0 - 1.0).abs() < 1e-9);
        assert!((result[9].0 - 5.0).abs() < 1e-9);
        // All densities should be positive
        for (_, density) in &result {
            assert!(*density >= 0.0);
        }
    }

    #[test]
    fn kde_empty_input() {
        assert!(compute_kde(&[], 10).is_empty());
    }

    #[test]
    fn kde_single_point() {
        // Single point should still produce valid output (range expanded)
        let result = compute_kde(&[5.0], 5);
        assert_eq!(result.len(), 5);
        for (_, density) in &result {
            assert!(*density >= 0.0);
        }
    }

    #[test]
    fn kde_identical_values() {
        let values = vec![3.0, 3.0, 3.0, 3.0];
        let result = compute_kde(&values, 5);
        assert_eq!(result.len(), 5);
        // Peak should be near the center (x=3.0)
        let max_density = result
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        assert!((max_density.0 - 3.0).abs() < 1.5);
    }

    #[test]
    fn kde_too_few_points_param() {
        assert!(compute_kde(&[1.0, 2.0], 0).is_empty());
        assert!(compute_kde(&[1.0, 2.0], 1).is_empty());
    }

    #[test]
    fn kde_nan_input() {
        let values = vec![f64::NAN, f64::NAN];
        let result = compute_kde(&values, 5);
        // min/max will be NaN → not finite → early return
        assert!(result.is_empty());
    }

    // --- compute_regression_points ---

    #[test]
    fn regression_with_slope() {
        let iters = vec![10.0, 20.0, 30.0];
        let measured = vec![100_000_000.0, 200_000_000.0, 300_000_000.0]; // ns
        let result = compute_regression_points(&iters, &measured, true);
        assert_eq!(result.len(), 3);
        // (iterations, total_time_ms)
        assert!((result[0].0 - 10.0).abs() < 1e-9);
        assert!((result[0].1 - 100.0).abs() < 1e-9); // 100M ns = 100 ms
        assert!((result[2].0 - 30.0).abs() < 1e-9);
        assert!((result[2].1 - 300.0).abs() < 1e-9);
    }

    #[test]
    fn regression_without_slope() {
        let iters = vec![100.0, 200.0];
        let measured = vec![500_000_000.0, 600_000_000.0]; // ns
        let result = compute_regression_points(&iters, &measured, false);
        assert_eq!(result.len(), 2);
        // (sample_index starting at 1, avg_iteration_time_ms)
        assert!((result[0].0 - 1.0).abs() < 1e-9);
        assert!((result[0].1 - 5.0).abs() < 1e-9); // 500M / 100 / 1M = 5ms
        assert!((result[1].0 - 2.0).abs() < 1e-9);
        assert!((result[1].1 - 3.0).abs() < 1e-9); // 600M / 200 / 1M = 3ms
    }

    #[test]
    fn regression_filters_non_finite() {
        let iters = vec![f64::NAN, 100.0];
        let measured = vec![500_000_000.0, 200_000_000.0];
        let result = compute_regression_points(&iters, &measured, true);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn regression_without_slope_filters_zero_iters() {
        let iters = vec![0.0, 100.0];
        let measured = vec![500_000_000.0, 100_000_000.0];
        let result = compute_regression_points(&iters, &measured, false);
        assert_eq!(result.len(), 1);
        assert!((result[0].0 - 2.0).abs() < 1e-9); // index 1 (0-based) → 2
    }

    #[test]
    fn regression_empty_input() {
        assert!(compute_regression_points(&[], &[], true).is_empty());
        assert!(compute_regression_points(&[], &[], false).is_empty());
    }
}
