use serde::{Deserialize, Serialize};

/// Unit of measurement for confidence interval values.
/// Serializes as the string representation for backward compatibility
/// with existing encoded URLs.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum TimeUnit {
    /// No unit (dimensionless values like R²)
    #[default]
    Dimensionless,
    /// Milliseconds (all timing values are normalized to ms)
    Milliseconds,
    /// Percentage (change between benchmark runs)
    Percent,
}

impl TimeUnit {
    pub fn is_dimensionless(&self) -> bool {
        *self == Self::Dimensionless
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Dimensionless => "",
            Self::Milliseconds => "ms",
            Self::Percent => "%",
        }
    }
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for TimeUnit {
    fn from(s: &str) -> Self {
        match s {
            "ms" => Self::Milliseconds,
            "%" => Self::Percent,
            _ => Self::Dimensionless,
        }
    }
}

impl Serialize for TimeUnit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TimeUnit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from(s.as_str()))
    }
}

/// Confidence interval with lower bound, point estimate, and upper bound.
/// Used for timing, change percentages, and statistical estimates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ConfidenceInterval {
    pub lower_bound: f64,
    pub estimate: f64,
    pub upper_bound: f64,
    /// Unit of measurement (e.g. Milliseconds, Percent, or Dimensionless)
    #[serde(default, skip_serializing_if = "TimeUnit::is_dimensionless")]
    pub unit: TimeUnit,
    /// Standard error of the estimate (from estimates.json)
    #[serde(default, skip_serializing_if = "is_zero")]
    pub standard_error: f64,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// Classification of change between benchmark runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ChangeResult {
    Improved,
    Regressed,
    #[default]
    NoChange,
}

/// Change statistics between the current and baseline runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ChangeStats {
    #[serde(default)]
    pub mean: ConfidenceInterval,
    #[serde(default)]
    pub median: ConfidenceInterval,
    #[serde(default)]
    pub result: ChangeResult,
    #[serde(default)]
    pub p_value: f64,
}

/// Throughput measurement for benchmarks that measure data processing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Throughput {
    pub per_iteration: f64,
    /// Unit type: "bytes", "elements", or "bits"
    pub unit: String,
}

/// Outlier classification from Criterion's statistical analysis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Outliers {
    pub total_measurements: u64,
    pub outlier_count: u64,
    #[serde(default)]
    pub mild_low: u64,
    #[serde(default)]
    pub mild_high: u64,
    #[serde(default)]
    pub severe_low: u64,
    #[serde(default)]
    pub severe_high: u64,
}

/// Metrics from a single Criterion.rs benchmark run.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CriterionMetrics {
    /// Benchmark name (e.g. "fib/20", "sort/1000")
    pub name: String,
    /// Primary timing confidence interval: [lower estimate upper]
    pub time: ConfidenceInterval,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<ConfidenceInterval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median: Option<ConfidenceInterval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median_abs_dev: Option<ConfidenceInterval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<ConfidenceInterval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slope: Option<ConfidenceInterval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<ChangeStats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throughput: Option<Throughput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outliers: Option<Outliers>,
    /// R² goodness of fit for linear regression (0.0 to 1.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r_squared: Option<ConfidenceInterval>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub iteration_count: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measured_values: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Box<CriterionMetrics>>,
}

// ---------------------------------------------------------------------------
// CLI stdout parser (cargo bench output)
// ---------------------------------------------------------------------------

/// Parse the text output from `cargo bench` (Criterion format) into metrics.
///
/// Example input:
/// ```text
/// fib/20                  time:   [1.9245 ms 1.9298 ms 1.9359 ms]
///                         change: [-0.5765% +0.2437% +1.1291%] (p = 0.59 > 0.05)
///                         No change in performance detected.
/// Found 3 outliers among 100 measurements (3.00%)
///   2 (2.00%) high mild
///   1 (1.00%) high severe
/// ```
pub fn parse_cli_output(output: &str) -> Vec<CriterionMetrics> {
    let mut results = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if let Some(metrics) = try_parse_benchmark_block(&lines, &mut i) {
            results.push(metrics);
        } else {
            i += 1;
        }
    }

    results
}

fn try_parse_benchmark_block(lines: &[&str], idx: &mut usize) -> Option<CriterionMetrics> {
    let line = lines[*idx];

    let time_pos = line.find("time:")?;
    let name = line[..time_pos].trim().to_string();
    if name.is_empty() {
        return None;
    }

    let time = parse_confidence_interval(&line[time_pos + 5..])?;
    let mut metrics = CriterionMetrics {
        name,
        time,
        ..Default::default()
    };

    *idx += 1;

    while *idx < lines.len() {
        let line = lines[*idx].trim();

        if let Some(stripped) = line.strip_prefix("thrpt:") {
            if let Some(ci) = parse_confidence_interval(stripped) {
                metrics.throughput = Some(Throughput {
                    per_iteration: ci.estimate,
                    unit: extract_throughput_unit(stripped),
                });
            }
            *idx += 1;
        } else if line.starts_with("change:") {
            if let Some(change) = parse_change_line(line, lines, idx) {
                metrics.change = Some(change);
            } else {
                *idx += 1;
            }
        } else if line.starts_with("Found") && line.contains("outliers among") {
            metrics.outliers = Some(parse_outliers(lines, idx));
        } else if line.contains("time:") && !line.starts_with("time:") {
            break;
        } else if !line.is_empty()
            && !line.starts_with("Benchmarking")
            && !line.starts_with("Warning:")
            && !line.starts_with("No change")
            && !line.starts_with("Performance has")
            && !line.starts_with("Change within")
            && !line.contains("regressed")
            && !line.contains("improved")
        {
            if is_new_benchmark_start(line) {
                break;
            }
            *idx += 1;
        } else {
            *idx += 1;
        }
    }

    Some(metrics)
}

fn is_new_benchmark_start(line: &str) -> bool {
    !line.starts_with(' ') && !line.starts_with('\t') && line.contains("time:")
}

/// Parse a confidence interval from text like `[1.9245 ms 1.9298 ms 1.9359 ms]`
fn parse_confidence_interval(text: &str) -> Option<ConfidenceInterval> {
    let text = text.trim();
    let bracket_start = text.find('[')?;
    let bracket_end = text.find(']')?;
    let inner = &text[bracket_start + 1..bracket_end];

    let parts: Vec<&str> = inner.split_whitespace().collect();

    if parts.len() >= 6 {
        let lower = parse_value_with_unit(parts[0], parts[1])?;
        let estimate = parse_value_with_unit(parts[2], parts[3])?;
        let upper = parse_value_with_unit(parts[4], parts[5])?;
        let unit = normalize_unit(parts[1]);
        Some(ConfidenceInterval {
            lower_bound: lower,
            estimate,
            upper_bound: upper,
            unit,
            ..Default::default()
        })
    } else if parts.len() >= 3 {
        let lower = parse_percentage_value(parts[0])?;
        let estimate = parse_percentage_value(parts[1])?;
        let upper = parse_percentage_value(parts[2])?;
        Some(ConfidenceInterval {
            lower_bound: lower,
            estimate,
            upper_bound: upper,
            unit: TimeUnit::Percent,
            ..Default::default()
        })
    } else {
        None
    }
}

fn parse_value_with_unit(value_str: &str, unit_str: &str) -> Option<f64> {
    let value: f64 = value_str.parse().ok()?;
    let multiplier = match unit_str.to_lowercase().as_str() {
        "ps" => 0.000_000_001,
        "ns" => 0.000_001,
        "us" | "µs" => 0.001,
        "s" => 1000.0,
        _ => 1.0,
    };
    Some(value * multiplier)
}

fn normalize_unit(unit: &str) -> TimeUnit {
    match unit.to_lowercase().as_str() {
        "ps" | "ns" | "us" | "µs" | "ms" | "s" => TimeUnit::Milliseconds,
        "%" => TimeUnit::Percent,
        _ => TimeUnit::Dimensionless,
    }
}

fn extract_throughput_unit(text: &str) -> String {
    let text = text.trim();
    let bracket_start = text.find('[').unwrap_or(0);
    let bracket_end = text.find(']').unwrap_or(text.len());
    let inner = &text[bracket_start + 1..bracket_end];
    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].strip_suffix("/s").unwrap_or(parts[1]).to_string()
    } else {
        String::new()
    }
}

fn parse_percentage_value(s: &str) -> Option<f64> {
    let s = s.trim().trim_end_matches('%').trim_start_matches('+');
    s.parse().ok()
}

fn parse_change_line(line: &str, lines: &[&str], idx: &mut usize) -> Option<ChangeStats> {
    let ci = parse_confidence_interval(&line[7..])?;

    // Extract p-value from "(p = 0.59 > 0.05)" or "(p = 0.01 < 0.05)"
    let p_value = line
        .find("(p = ")
        .and_then(|start| {
            let rest = &line[start + 5..];
            rest.split_whitespace().next()?.parse::<f64>().ok()
        })
        .unwrap_or(0.0);

    *idx += 1;

    // Next line should describe the change result
    let result = if *idx < lines.len() {
        let result_line = lines[*idx].trim().to_lowercase();
        let r = if result_line.contains("improved") || result_line.contains("decrease") {
            ChangeResult::Improved
        } else if result_line.contains("regressed") || result_line.contains("increase") {
            ChangeResult::Regressed
        } else {
            ChangeResult::NoChange
        };
        *idx += 1;
        r
    } else {
        ChangeResult::NoChange
    };

    Some(ChangeStats {
        mean: ci.clone(),
        median: ci,
        result,
        p_value,
    })
}

fn parse_outliers(lines: &[&str], idx: &mut usize) -> Outliers {
    let line = lines[*idx];
    let mut outliers = Outliers::default();

    // Parse "Found 3 outliers among 100 measurements (3.00%)"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 5 {
        outliers.outlier_count = parts[1].parse().unwrap_or(0);
        outliers.total_measurements = parts[4].parse().unwrap_or(0);
    }

    *idx += 1;

    // Parse subsequent outlier classification lines
    while *idx < lines.len() {
        let oline = lines[*idx].trim();
        if oline.is_empty()
            || (!oline.starts_with(|c: char| c.is_ascii_digit()) && !oline.starts_with(' '))
        {
            break;
        }

        let parts: Vec<&str> = oline.split_whitespace().collect();
        if parts.len() >= 3 {
            let count: u64 = parts[0].parse().unwrap_or(0);
            let classification = parts[2..].join(" ");
            match classification.as_str() {
                "high mild" => outliers.mild_high = count,
                "high severe" => outliers.severe_high = count,
                "low mild" => outliers.mild_low = count,
                "low severe" => outliers.severe_low = count,
                _ => {}
            }
        }
        *idx += 1;
    }

    outliers
}

// ---------------------------------------------------------------------------
// JSON parser (cargo-criterion --message-format=json)
// ---------------------------------------------------------------------------

/// Parse JSON output from `cargo-criterion --message-format=json`.
/// Each line is a separate JSON object; we only care about "benchmark-complete" messages.
pub fn parse_json_output(output: &str) -> Vec<CriterionMetrics> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            parse_json_message(line)
        })
        .collect()
}

fn parse_json_message(line: &str) -> Option<CriterionMetrics> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;

    let reason = obj.get("reason")?.as_str()?;
    if reason != "benchmark-complete" {
        return None;
    }

    let name = obj.get("id")?.as_str()?.to_string();
    let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("ns");

    let typical = parse_json_ci(obj.get("typical")?, unit)?;
    let mean = parse_json_ci(obj.get("mean")?, unit);
    let median = parse_json_ci(obj.get("median")?, unit);
    let median_abs_dev = parse_json_ci(obj.get("median_abs_dev")?, unit);
    let slope = obj.get("slope").and_then(|s| parse_json_ci(s, unit));
    let std_dev = obj.get("std_dev").and_then(|s| parse_json_ci(s, unit));

    let change = obj.get("change").and_then(parse_json_change);

    let throughput = obj.get("throughput").and_then(|t| {
        let arr = t.as_array()?;
        let first = arr.first()?.as_object()?;
        Some(Throughput {
            per_iteration: first.get("per_iteration")?.as_f64()?,
            unit: first.get("unit")?.as_str()?.to_string(),
        })
    });

    let iteration_count = obj
        .get("iteration_count")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(serde_json::Value::as_f64).collect())
        .unwrap_or_default();

    let measured_values = obj
        .get("measured_values")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(serde_json::Value::as_f64).collect())
        .unwrap_or_default();

    Some(CriterionMetrics {
        name,
        time: typical,
        mean,
        median,
        median_abs_dev,
        std_dev,
        slope,
        change,
        throughput,
        outliers: None,
        r_squared: None,
        iteration_count,
        measured_values,
        baseline: None,
    })
}

fn parse_json_ci(value: &serde_json::Value, unit: &str) -> Option<ConfidenceInterval> {
    let obj = value.as_object()?;
    let lower = obj.get("lower_bound")?.as_f64()?;
    let estimate = obj.get("estimate")?.as_f64()?;
    let upper = obj.get("upper_bound")?.as_f64()?;

    let multiplier = match unit {
        "us" | "µs" => 0.001,
        "ms" => 1.0,
        "s" => 1000.0,
        _ => 0.000_001,
    };

    Some(ConfidenceInterval {
        lower_bound: lower * multiplier,
        estimate: estimate * multiplier,
        upper_bound: upper * multiplier,
        unit: TimeUnit::Milliseconds,
        ..Default::default()
    })
}

fn parse_json_change(value: &serde_json::Value) -> Option<ChangeStats> {
    let obj = value.as_object()?;

    let mean = obj
        .get("mean")
        .and_then(|m| {
            let mobj = m.as_object()?;
            Some(ConfidenceInterval {
                estimate: mobj.get("estimate")?.as_f64()? * 100.0,
                unit: TimeUnit::Percent,
                ..Default::default()
            })
        })
        .unwrap_or_default();

    let median = obj
        .get("median")
        .and_then(|m| {
            let mobj = m.as_object()?;
            Some(ConfidenceInterval {
                estimate: mobj.get("estimate")?.as_f64()? * 100.0,
                unit: TimeUnit::Percent,
                ..Default::default()
            })
        })
        .unwrap_or_default();

    let result = obj
        .get("change")
        .and_then(|c| c.as_str())
        .map_or(ChangeResult::NoChange, |s| match s {
            "Improved" => ChangeResult::Improved,
            "Regressed" => ChangeResult::Regressed,
            _ => ChangeResult::NoChange,
        });

    Some(ChangeStats {
        mean,
        median,
        result,
        p_value: 0.0,
    })
}

// ---------------------------------------------------------------------------
// Linear regression: slope (ns/iter) and R² from raw samples
// ---------------------------------------------------------------------------

fn compute_linear_regression(
    iters: &[f64],
    times_ns: &[f64],
) -> (Option<ConfidenceInterval>, Option<ConfidenceInterval>) {
    let n = iters.len();
    if n < 2 {
        return (None, None);
    }

    #[allow(clippy::cast_precision_loss)]
    let n_f = n as f64;
    let sum_x: f64 = iters.iter().sum();
    let sum_y: f64 = times_ns.iter().sum();
    let dot_product: f64 = iters.iter().zip(times_ns.iter()).map(|(x, y)| x * y).sum();
    let sum_x_squared: f64 = iters.iter().map(|x| x * x).sum();

    let denom = n_f * sum_x_squared - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return (None, None);
    }

    let slope_ns_per_iter = (n_f * dot_product - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope_ns_per_iter * sum_x) / n_f;

    let ss_res: f64 = iters
        .iter()
        .zip(times_ns.iter())
        .map(|(x, y)| {
            let predicted = slope_ns_per_iter * x + intercept;
            (y - predicted).powi(2)
        })
        .sum();
    let mean_y = sum_y / n_f;
    let ss_tot: f64 = times_ns.iter().map(|y| (y - mean_y).powi(2)).sum();

    let r_squared_value = if ss_tot.abs() < f64::EPSILON {
        1.0
    } else {
        1.0 - ss_res / ss_tot
    };

    let ns_to_ms = 0.000_001;
    let slope_ms = slope_ns_per_iter * ns_to_ms;

    let std_error = if n > 2 {
        let mse = ss_res / (n_f - 2.0);
        let x_var = sum_x_squared - sum_x * sum_x / n_f;
        (mse / x_var).sqrt() * ns_to_ms
    } else {
        0.0
    };

    let slope_ci = ConfidenceInterval {
        lower_bound: slope_ms - 1.96 * std_error,
        estimate: slope_ms,
        upper_bound: slope_ms + 1.96 * std_error,
        unit: TimeUnit::Milliseconds,
        standard_error: std_error,
    };

    let r_squared_ci = ConfidenceInterval {
        lower_bound: r_squared_value,
        estimate: r_squared_value,
        upper_bound: r_squared_value,
        ..Default::default()
    };

    (Some(slope_ci), Some(r_squared_ci))
}

// ---------------------------------------------------------------------------
// Raw sample.json parser (target/criterion/$NAME/new/sample.json)
// ---------------------------------------------------------------------------

#[allow(clippy::cast_precision_loss)]
pub fn parse_sample_json(output: &str) -> Option<CriterionMetrics> {
    let value: serde_json::Value = serde_json::from_str(output).ok()?;
    let obj = value.as_object()?;

    let iters = obj.get("iters")?.as_array()?;
    let times = obj.get("times")?.as_array()?;

    if iters.is_empty() || iters.len() != times.len() {
        return None;
    }

    let per_iter_ns: Vec<f64> = iters
        .iter()
        .zip(times.iter())
        .filter_map(|(i, t)| {
            let iter_count = i.as_f64()?;
            let time_ns = t.as_f64()?;
            if iter_count > 0.0 {
                Some(time_ns / iter_count)
            } else {
                None
            }
        })
        .collect();

    if per_iter_ns.is_empty() {
        return None;
    }

    let mean_nanoseconds = per_iter_ns.iter().sum::<f64>() / per_iter_ns.len() as f64;
    let median_nanoseconds = {
        let mut sorted = per_iter_ns.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            f64::midpoint(sorted[mid - 1], sorted[mid])
        } else {
            sorted[mid]
        }
    };
    let std_dev_nanoseconds = {
        let variance = per_iter_ns
            .iter()
            .map(|x| (x - mean_nanoseconds).powi(2))
            .sum::<f64>()
            / per_iter_ns.len() as f64;
        variance.sqrt()
    };

    let ns_to_ms = 0.000_001;
    let mean_milliseconds = mean_nanoseconds * ns_to_ms;
    let median_milliseconds = median_nanoseconds * ns_to_ms;
    let std_dev_milliseconds = std_dev_nanoseconds * ns_to_ms;

    let iteration_count: Vec<f64> = iters.iter().filter_map(serde_json::Value::as_f64).collect();
    let measured_values: Vec<f64> = times.iter().filter_map(serde_json::Value::as_f64).collect();

    let (slope, r_squared) = compute_linear_regression(&iteration_count, &measured_values);

    let time = ConfidenceInterval {
        lower_bound: mean_milliseconds - std_dev_milliseconds,
        estimate: mean_milliseconds,
        upper_bound: mean_milliseconds + std_dev_milliseconds,
        unit: TimeUnit::Milliseconds,
        ..Default::default()
    };

    Some(CriterionMetrics {
        name: "benchmark".to_string(),
        time,
        mean: Some(ConfidenceInterval {
            lower_bound: mean_milliseconds - std_dev_milliseconds,
            estimate: mean_milliseconds,
            upper_bound: mean_milliseconds + std_dev_milliseconds,
            unit: TimeUnit::Milliseconds,
            ..Default::default()
        }),
        median: Some(ConfidenceInterval {
            lower_bound: median_milliseconds - std_dev_milliseconds,
            estimate: median_milliseconds,
            upper_bound: median_milliseconds + std_dev_milliseconds,
            unit: TimeUnit::Milliseconds,
            ..Default::default()
        }),
        std_dev: Some(ConfidenceInterval {
            estimate: std_dev_milliseconds,
            unit: TimeUnit::Milliseconds,
            ..Default::default()
        }),
        slope,
        r_squared,
        median_abs_dev: None,
        change: None,
        throughput: None,
        outliers: None,
        iteration_count,
        measured_values,
        baseline: None,
    })
}

// ---------------------------------------------------------------------------
// Auto-detection
// ---------------------------------------------------------------------------

pub fn try_parse(output: &str) -> Option<Vec<CriterionMetrics>> {
    if is_criterion_message_json(output) {
        let results = parse_json_output(output);
        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    } else if is_criterion_sample_json(output) {
        parse_sample_json(output).map(|m| vec![m])
    } else if is_criterion_cli(output) {
        let results = parse_cli_output(output);
        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    } else {
        None
    }
}

fn is_criterion_message_json(output: &str) -> bool {
    output
        .lines()
        .find(|l| !l.trim().is_empty())
        .is_some_and(|first_line| {
            first_line.trim().starts_with('{')
                && (first_line.contains("\"benchmark-complete\"")
                    || first_line.contains("\"group-complete\"")
                    || first_line.contains("\"reason\""))
        })
}

fn is_criterion_sample_json(output: &str) -> bool {
    let trimmed = output.trim();
    trimmed.starts_with('{')
        && trimmed.contains("\"sampling_mode\"")
        && trimmed.contains("\"iters\"")
        && trimmed.contains("\"times\"")
}

fn is_criterion_cli(output: &str) -> bool {
    let text = output.to_lowercase();
    text.contains("time:")
        && text.contains('[')
        && text.contains(']')
        && (text.contains("benchmarking")
            || text.contains("change:")
            || (text.contains("found") && text.contains("outliers")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_float_eq(a: f64, b: f64) {
        const EPSILON: f64 = 1e-4;
        assert!(
            (a - b).abs() < EPSILON,
            "Expected {a} to be approximately equal to {b}"
        );
    }

    const CLI_OUTPUT_SIMPLE: &str = r"
Benchmarking fib/20
Benchmarking fib/20: Warming up for 3.0000 s
Benchmarking fib/20: Collecting 100 samples in estimated 5.0159 s (2600 iterations)
Benchmarking fib/20: Analyzing
fib/20                  time:   [1.9245 ms 1.9298 ms 1.9359 ms]
                        change: [-0.5765% +0.2437% +1.1291%] (p = 0.59 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
";

    #[test]
    fn test_parse_cli_simple() {
        let results = parse_cli_output(CLI_OUTPUT_SIMPLE);
        assert_eq!(results.len(), 1);

        let m = &results[0];
        assert_eq!(m.name, "fib/20");
        assert_float_eq(m.time.lower_bound, 1.9245);
        assert_float_eq(m.time.estimate, 1.9298);
        assert_float_eq(m.time.upper_bound, 1.9359);
        assert_eq!(m.time.unit, TimeUnit::Milliseconds);

        let change = m.change.as_ref().unwrap();
        assert_float_eq(change.mean.lower_bound, -0.5765);
        assert_float_eq(change.mean.estimate, 0.2437);
        assert_float_eq(change.mean.upper_bound, 1.1291);
        assert_float_eq(change.p_value, 0.59);
        assert_eq!(change.result, ChangeResult::NoChange);

        let outliers = m.outliers.as_ref().unwrap();
        assert_eq!(outliers.outlier_count, 3);
        assert_eq!(outliers.total_measurements, 100);
        assert_eq!(outliers.mild_high, 2);
        assert_eq!(outliers.severe_high, 1);
    }

    const CLI_OUTPUT_MULTIPLE: &str = r"
fib/10                  time:   [50.123 us 51.456 us 52.789 us]
                        change: [-2.1234% -1.5678% -0.9012%] (p = 0.01 < 0.05)
                        Performance has improved.

fib/20                  time:   [1.9245 ms 1.9298 ms 1.9359 ms]
                        change: [+1.234% +2.567% +3.890%] (p = 0.02 < 0.05)
                        Performance has regressed.
";

    #[test]
    fn test_parse_cli_multiple() {
        let results = parse_cli_output(CLI_OUTPUT_MULTIPLE);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].name, "fib/10");
        // 50.123 us = 0.050123 ms
        assert_float_eq(results[0].time.lower_bound, 0.050_123);
        assert_float_eq(results[0].time.estimate, 0.051_456);
        assert_float_eq(results[0].time.upper_bound, 0.052_789);

        let change0 = results[0].change.as_ref().unwrap();
        assert_eq!(change0.result, ChangeResult::Improved);

        assert_eq!(results[1].name, "fib/20");
        assert_float_eq(results[1].time.lower_bound, 1.9245);
        assert_float_eq(results[1].time.estimate, 1.9298);

        let change1 = results[1].change.as_ref().unwrap();
        assert_eq!(change1.result, ChangeResult::Regressed);
    }

    const CLI_OUTPUT_NANOSECONDS: &str = r"
sort/small              time:   [245.67 ns 248.90 ns 252.13 ns]
";

    #[test]
    fn test_parse_cli_nanoseconds() {
        let results = parse_cli_output(CLI_OUTPUT_NANOSECONDS);
        assert_eq!(results.len(), 1);

        let m = &results[0];
        assert_eq!(m.name, "sort/small");
        // 245.67 ns = 0.00024567 ms
        assert_float_eq(m.time.lower_bound, 0.000_245_67);
        assert_float_eq(m.time.estimate, 0.000_248_90);
        assert_float_eq(m.time.upper_bound, 0.000_252_13);
    }

    const JSON_OUTPUT: &str = r#"{"reason":"benchmark-complete","id":"norm","report_directory":"target/criterion/reports/norm","iteration_count":[30,60,90],"measured_values":[124200.0,248400.0,372600.0],"unit":"ns","throughput":[{"per_iteration":1024,"unit":"elements"}],"typical":{"estimate":3419.49,"lower_bound":3375.24,"upper_bound":3465.46,"unit":"ns"},"mean":{"estimate":3419.49,"lower_bound":3375.24,"upper_bound":3465.46,"unit":"ns"},"median":{"estimate":3400.00,"lower_bound":3360.00,"upper_bound":3440.00,"unit":"ns"},"median_abs_dev":{"estimate":50.0,"lower_bound":40.0,"upper_bound":60.0,"unit":"ns"},"slope":{"estimate":3410.0,"lower_bound":3370.0,"upper_bound":3450.0,"unit":"ns"},"change":{"mean":{"estimate":0.014,"unit":"%"},"median":{"estimate":0.012,"unit":"%"},"change":"NoChange"}}"#;

    #[test]
    fn test_parse_json() {
        let results = parse_json_output(JSON_OUTPUT);
        assert_eq!(results.len(), 1);

        let m = &results[0];
        assert_eq!(m.name, "norm");
        // 3419.49 ns = 0.00341949 ms
        assert_float_eq(m.time.estimate, 0.003_419_49);
        assert_float_eq(m.time.lower_bound, 0.003_375_24);
        assert_float_eq(m.time.upper_bound, 0.003_465_46);

        assert!(m.mean.is_some());
        assert!(m.median.is_some());
        assert!(m.median_abs_dev.is_some());
        assert!(m.slope.is_some());

        let throughput = m.throughput.as_ref().unwrap();
        assert_float_eq(throughput.per_iteration, 1024.0);
        assert_eq!(throughput.unit, "elements");

        let change = m.change.as_ref().unwrap();
        assert_float_eq(change.mean.estimate, 1.4); // 0.014 * 100
        assert_eq!(change.result, ChangeResult::NoChange);

        assert_eq!(m.iteration_count, vec![30.0, 60.0, 90.0]);
        assert_eq!(m.measured_values, vec![124_200.0, 248_400.0, 372_600.0]);
    }

    #[test]
    fn test_auto_detect_cli() {
        assert!(is_criterion_cli(CLI_OUTPUT_SIMPLE));
        assert!(!is_criterion_message_json(CLI_OUTPUT_SIMPLE));
        assert!(try_parse(CLI_OUTPUT_SIMPLE).is_some());
    }

    #[test]
    fn test_auto_detect_json() {
        assert!(is_criterion_message_json(JSON_OUTPUT));
        assert!(!is_criterion_cli(JSON_OUTPUT));
        assert!(try_parse(JSON_OUTPUT).is_some());
    }

    #[test]
    fn test_auto_detect_wrk() {
        let wrk_output = r"
Running 10s test @ http://localhost:8080
  2 threads and 100 connections
  Thread Stats   Avg      stddev     Max   +/- stddev
    Latency   125.12ms   25.31ms 450.00ms   90.12%
";
        assert!(!is_criterion_message_json(wrk_output));
        assert!(try_parse(wrk_output).is_none() || !is_criterion_cli(wrk_output));
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_cli_output("").is_empty());
        assert!(parse_json_output("").is_empty());
        assert!(try_parse("").is_none());
    }

    #[test]
    fn test_parse_sample_json() {
        let input = include_str!("../../criterion_sample.json");
        let result = parse_sample_json(input).expect("should parse sample.json");

        assert_eq!(result.name, "benchmark");
        assert_eq!(result.iteration_count.len(), 100);
        assert_eq!(result.measured_values.len(), 100);

        assert!(result.time.estimate > 0.0);
        assert!(result.time.lower_bound < result.time.estimate);
        assert!(result.time.upper_bound > result.time.estimate);
        assert_eq!(result.time.unit, TimeUnit::Milliseconds);

        let mean = result.mean.as_ref().unwrap();
        assert!(mean.estimate > 0.0);

        let median = result.median.as_ref().unwrap();
        assert!(median.estimate > 0.0);
        assert!(median.estimate <= mean.estimate);

        let std_dev = result.std_dev.as_ref().unwrap();
        assert!(std_dev.estimate > 0.0);
    }

    #[test]
    fn test_auto_detect_sample_json() {
        let input = include_str!("../../criterion_sample.json");
        assert!(is_criterion_sample_json(input));
        let results = try_parse(input).expect("should auto-detect sample.json");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "benchmark");
    }

    #[test]
    fn test_sample_json_roundtrip_serialize() {
        let input = include_str!("../../criterion_sample.json");
        let results = try_parse(input).unwrap();
        let bench = &results[0];

        let json = serde_json::to_string(bench).expect("should serialize");
        let deserialized: CriterionMetrics =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(bench.name, deserialized.name);
        assert_eq!(
            bench.iteration_count.len(),
            deserialized.iteration_count.len()
        );
    }
}
