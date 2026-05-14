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
