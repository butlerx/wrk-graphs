use crate::parser::criterion::{ConfidenceInterval, CriterionMetrics, Throughput};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CriterionStatsTableProps {
    pub metrics: CriterionMetrics,
}

#[derive(Clone)]
enum StatValue {
    Interval(ConfidenceInterval),
    Throughput(Throughput),
}

#[function_component(CriterionStatsTable)]
pub fn criterion_stats_table(props: &CriterionStatsTableProps) -> Html {
    let metrics = &props.metrics;

    let mut rows: Vec<(String, StatValue)> = Vec::new();

    if let Some(slope) = &metrics.slope {
        rows.push(("Slope".to_string(), StatValue::Interval(slope.clone())));
    }
    if let Some(r_squared) = &metrics.r_squared {
        rows.push(("R²".to_string(), StatValue::Interval(r_squared.clone())));
    }
    if let Some(mean) = &metrics.mean {
        rows.push(("Mean".to_string(), StatValue::Interval(mean.clone())));
    }
    if let Some(std_dev) = &metrics.std_dev {
        rows.push((
            "Std. Dev.".to_string(),
            StatValue::Interval(std_dev.clone()),
        ));
    }
    if let Some(median) = &metrics.median {
        rows.push(("Median".to_string(), StatValue::Interval(median.clone())));
    }
    if let Some(mad) = &metrics.median_abs_dev {
        rows.push(("MAD".to_string(), StatValue::Interval(mad.clone())));
    }
    if let Some(throughput) = &metrics.throughput {
        rows.push((
            "Throughput".to_string(),
            StatValue::Throughput(throughput.clone()),
        ));
    }

    html! {
        <div class="criterion-stats-table">
            <div class="criterion-confidence">{ "Confidence level: 95%" }</div>
            <table>
                <thead>
                    <tr>
                        <th>{ "Statistic" }</th>
                        <th>{ "Lower Bound" }</th>
                        <th>{ "Estimate" }</th>
                        <th>{ "Upper Bound" }</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        for rows.iter().map(|(label, value)| {
                            let (lower, estimate, upper) = format_stat_row(label, value);
                            html! {
                                <tr>
                                    <td>{ label }</td>
                                    <td>{ lower }</td>
                                    <td>{ estimate }</td>
                                    <td>{ upper }</td>
                                </tr>
                            }
                        })
                    }
                </tbody>
            </table>
        </div>
    }
}

fn format_stat_row(label: &str, value: &StatValue) -> (String, String, String) {
    match value {
        StatValue::Interval(ci) if label == "R²" => (
            format!("{:.7}", ci.lower_bound),
            format!("{:.7}", ci.estimate),
            format!("{:.7}", ci.upper_bound),
        ),
        StatValue::Interval(ci) => (
            format_timing(ci.lower_bound),
            format_timing(ci.estimate),
            format_timing(ci.upper_bound),
        ),
        StatValue::Throughput(throughput) => {
            let estimate = format_throughput(throughput.per_iteration, &throughput.unit);
            (estimate.clone(), estimate.clone(), estimate)
        }
    }
}

fn format_timing(value_ms: f64) -> String {
    if value_ms < 0.001 {
        format!("{:.2} ns", value_ms * 1_000_000.0)
    } else if value_ms < 1.0 {
        format!("{:.2} µs", value_ms * 1000.0)
    } else if value_ms >= 1000.0 {
        format!("{:.3} s", value_ms / 1000.0)
    } else {
        format!("{value_ms:.4} ms")
    }
}

fn format_throughput(value: f64, unit: &str) -> String {
    format!("{value:.2} {unit}/iter")
}
