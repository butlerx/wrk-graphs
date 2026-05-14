use super::{
    CriterionComparison, CriterionIterationTimesChart, CriterionPdfChart, CriterionRegressionChart,
    CriterionStatDistributionChart, CriterionStatsTable,
};
use crate::{
    components::MetricPanel,
    parser::criterion::{ChangeResult, ConfidenceInterval, CriterionMetrics},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CriterionBenchmarkProps {
    pub metrics: CriterionMetrics,
}

fn format_latency(value: f64) -> String {
    if value < 0.001 {
        format!("{:.2} ns", value * 1_000_000.0)
    } else if value < 1.0 {
        format!("{:.2} \u{00b5}s", value * 1000.0)
    } else if value >= 1000.0 {
        format!("{:.3} s", value / 1000.0)
    } else {
        format!("{value:.4} ms")
    }
}

#[function_component(CriterionBenchmark)]
pub fn criterion_benchmark(props: &CriterionBenchmarkProps) -> Html {
    let bench = &props.metrics;

    let change_html = render_change_panel(bench);
    let outliers_html = render_outliers_panel(bench);

    let throughput_html = bench.throughput.as_ref().map(|tp| {
        html! {
            <MetricPanel
                class="panel-throughput"
                value={format!("{:.2} {}/iter", tp.per_iteration, tp.unit)}
                label="Throughput"
            />
        }
    });

    let has_samples = !bench.iteration_count.is_empty() && !bench.measured_values.is_empty();

    let iteration_times_html = if has_samples && bench.slope.is_none() {
        Some(html! {
            <div class="chart-panel">
                <h4 class="chart-title">{ "Iteration Times" }</h4>
                <CriterionIterationTimesChart
                    iteration_count={bench.iteration_count.clone()}
                    measured_values={bench.measured_values.clone()}
                />
                <p class="chart-description">
                    { "This chart shows the time taken by each iteration of the benchmark. This is used when the benchmark does not do linear regression." }
                </p>
            </div>
        })
    } else {
        None
    };

    let stat_distributions_html = render_stat_distributions(bench);

    html! {
        <div class="criterion-benchmark">
            <h3 class="benchmark-name">{ &bench.name }</h3>
            <div class="benchmark-metrics">
                <div class="metric-panel panel-time">
                    <h3>{ "Time" }</h3>
                    <div class="metric-content">
                        <div class="main-value">{ format_latency(bench.time.estimate) }</div>
                        <div class="confidence-interval">
                            { format!("[{} .. {}]", format_latency(bench.time.lower_bound), format_latency(bench.time.upper_bound)) }
                        </div>
                    </div>
                </div>
                { for change_html }
                { for outliers_html }
                { for throughput_html }
            </div>
            <div class="benchmark-stats">
                <CriterionStatsTable metrics={bench.clone()} />
            </div>
            <div class="benchmark-charts">
                if has_samples {
                    <div class="chart-row">
                        <div class="chart-panel">
                            <h4 class="chart-title">{ "Probability Density Function" }</h4>
                            <CriterionPdfChart
                                iteration_count={bench.iteration_count.clone()}
                                measured_values={bench.measured_values.clone()}
                            />
                            <p class="chart-description">
                                { "This chart shows an estimate of the probability density function of time per iteration." }
                            </p>
                        </div>
                        if bench.slope.is_some() {
                            <div class="chart-panel">
                                <h4 class="chart-title">{ "Linear Regression" }</h4>
                                <CriterionRegressionChart
                                    iteration_count={bench.iteration_count.clone()}
                                    measured_values={bench.measured_values.clone()}
                                    slope={bench.slope.clone()}
                                />
                                <p class="chart-description">
                                    { "This chart shows the linear regression of total time vs. number of iterations, with the confidence interval shaded." }
                                </p>
                            </div>
                        }
                        { for iteration_times_html }
                    </div>
                }
                { for stat_distributions_html }
            </div>
            <CriterionComparison metrics={bench.clone()} />
        </div>
    }
}

fn render_change_panel(bench: &CriterionMetrics) -> Option<Html> {
    bench.change.as_ref().map(|change| {
        let change_class = match change.result {
            ChangeResult::Improved => "change-improved",
            ChangeResult::Regressed => "change-regressed",
            ChangeResult::NoChange => "change-none",
        };
        let change_label = match change.result {
            ChangeResult::Improved => "Improved",
            ChangeResult::Regressed => "Regressed",
            ChangeResult::NoChange => "No change",
        };
        html! {
            <div class={classes!("metric-panel", "panel-change", change_class)}>
                <h3>{ "Change" }</h3>
                <div class="metric-content">
                    <div class="main-value">{ format!("{:+.3}%", change.mean.estimate) }</div>
                    <div class="metric-label">{ change_label }</div>
                    <div class="metric-row">
                        <div class="metric-label">{ "Mean" }</div>
                        <div class="metric-value">
                            { format!("[{:+.3}% .. {:+.3}%]", change.mean.lower_bound, change.mean.upper_bound) }
                        </div>
                    </div>
                    <div class="metric-row">
                        <div class="metric-label">{ "Median" }</div>
                        <div class="metric-value">
                            { format!("{:+.3}%", change.median.estimate) }
                        </div>
                    </div>
                    if change.p_value > 0.0 {
                        <div class="metric-row">
                            <div class="metric-label">{ "p-value" }</div>
                            <div class="metric-value">{ format!("{:.4}", change.p_value) }</div>
                        </div>
                    }
                </div>
            </div>
        }
    })
}

fn render_outliers_panel(bench: &CriterionMetrics) -> Option<Html> {
    bench.outliers.as_ref().map(|outliers| {
        html! {
            <div class="metric-panel panel-outliers">
                <h3>{ "Outliers" }</h3>
                <div class="metric-content">
                    <div class="metric-row">
                        <div class="metric-label">{ "Total" }</div>
                        <div class="metric-value">
                            { format!("{} / {}", outliers.outlier_count, outliers.total_measurements) }
                        </div>
                    </div>
                    if outliers.mild_high > 0 {
                        <div class="metric-row">
                            <div class="metric-label">{ "High mild" }</div>
                            <div class="metric-value">{ outliers.mild_high }</div>
                        </div>
                    }
                    if outliers.severe_high > 0 {
                        <div class="metric-row">
                            <div class="metric-label">{ "High severe" }</div>
                            <div class="metric-value">{ outliers.severe_high }</div>
                        </div>
                    }
                    if outliers.mild_low > 0 {
                        <div class="metric-row">
                            <div class="metric-label">{ "Low mild" }</div>
                            <div class="metric-value">{ outliers.mild_low }</div>
                        </div>
                    }
                    if outliers.severe_low > 0 {
                        <div class="metric-row">
                            <div class="metric-label">{ "Low severe" }</div>
                            <div class="metric-value">{ outliers.severe_low }</div>
                        </div>
                    }
                </div>
            </div>
        }
    })
}

fn render_stat_distributions(bench: &CriterionMetrics) -> Option<Html> {
    let stats: Vec<(&str, &ConfidenceInterval)> = [
        ("Slope", bench.slope.as_ref()),
        ("Mean", bench.mean.as_ref()),
        ("Median", bench.median.as_ref()),
        ("Std. Dev.", bench.std_dev.as_ref()),
        ("MAD", bench.median_abs_dev.as_ref()),
    ]
    .into_iter()
    .filter_map(|(label, ci)| ci.map(|c| (label, c)))
    .collect();

    if stats.is_empty() {
        return None;
    }

    Some(html! {
        <div class="stat-distributions">
            <h4 class="stat-distributions-title">{ "Additional Statistics" }</h4>
            <p class="stat-distributions-description">
                { "These charts show the bootstrap distribution of each statistic. The shaded region represents the confidence interval for the estimate." }
            </p>
            <div class="stat-distributions-grid">
                { for stats.into_iter().map(|(label, ci)| html! {
                    <div class="stat-distribution-panel">
                        <CriterionStatDistributionChart
                            ci={ci.clone()}
                            label={label}
                        />
                    </div>
                }) }
            </div>
        </div>
    })
}
