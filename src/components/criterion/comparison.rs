use super::{
    CriterionPdfComparisonChart, CriterionRegressionComparisonChart, CriterionStatDistributionChart,
};
use crate::parser::criterion::{ChangeResult, ChangeStats, CriterionMetrics};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CriterionComparisonProps {
    pub metrics: CriterionMetrics,
}

#[function_component(CriterionComparison)]
pub fn criterion_comparison(props: &CriterionComparisonProps) -> Html {
    let bench = &props.metrics;
    let Some(baseline) = bench.baseline.as_ref() else {
        return html! {};
    };

    let has_current_samples =
        !bench.iteration_count.is_empty() && !bench.measured_values.is_empty();
    let has_baseline_samples =
        !baseline.iteration_count.is_empty() && !baseline.measured_values.is_empty();

    let comparison_charts = if has_current_samples && has_baseline_samples {
        let pdf_chart = html! {
            <div class="chart-panel">
                <h4 class="chart-title">{ "PDF Comparison" }</h4>
                <CriterionPdfComparisonChart
                    iteration_count={bench.iteration_count.clone()}
                    measured_values={bench.measured_values.clone()}
                    baseline_iteration_count={baseline.iteration_count.clone()}
                    baseline_measured_values={baseline.measured_values.clone()}
                />
                <p class="chart-description">
                    { "This chart shows the estimated probability density functions of the current and baseline benchmarks overlaid, highlighting shifts in performance." }
                </p>
            </div>
        };

        let regression_chart = if bench.slope.is_some() || baseline.slope.is_some() {
            Some(html! {
                <div class="chart-panel">
                    <h4 class="chart-title">{ "Regression Comparison" }</h4>
                    <CriterionRegressionComparisonChart
                        iteration_count={bench.iteration_count.clone()}
                        measured_values={bench.measured_values.clone()}
                        slope={bench.slope.clone()}
                        baseline_iteration_count={baseline.iteration_count.clone()}
                        baseline_measured_values={baseline.measured_values.clone()}
                        baseline_slope={baseline.slope.clone()}
                    />
                    <p class="chart-description">
                        { "This chart compares the linear regressions of the current and baseline benchmarks. A steeper slope indicates slower performance per iteration." }
                    </p>
                </div>
            })
        } else {
            None
        };

        Some(html! { <div class="comparison-charts">{ pdf_chart }{ for regression_chart }</div> })
    } else {
        None
    };

    let change_distributions = render_change_distributions(bench.change.as_ref());
    let change_table = render_change_table(bench.change.as_ref());

    html! {
        <div class="comparison-section">
            <h4 class="comparison-title">{ "Change Since Previous" }</h4>
            { for comparison_charts }
            { for change_table }
            { for change_distributions }
        </div>
    }
}

fn render_change_distributions(change: Option<&ChangeStats>) -> Option<Html> {
    let change = change?;

    let has_mean = change.mean.lower_bound != 0.0
        || change.mean.estimate != 0.0
        || change.mean.upper_bound != 0.0;
    let has_median = change.median.lower_bound != 0.0
        || change.median.estimate != 0.0
        || change.median.upper_bound != 0.0;

    if !has_mean && !has_median {
        return None;
    }

    Some(html! {
        <div class="change-distributions">
            <h4 class="change-distributions-title">{ "Change Distributions" }</h4>
            <p class="change-distributions-description">
                { "These charts show the bootstrap distribution of the percentage change in each statistic relative to the previous benchmark run." }
            </p>
            <div class="change-distributions-grid">
                if has_mean {
                    <div class="stat-distribution-panel">
                        <CriterionStatDistributionChart
                            ci={change.mean.clone()}
                            label="Mean (change)"
                        />
                    </div>
                }
                if has_median {
                    <div class="stat-distribution-panel">
                        <CriterionStatDistributionChart
                            ci={change.median.clone()}
                            label="Median (change)"
                        />
                    </div>
                }
            </div>
        </div>
    })
}

fn render_change_table(change: Option<&ChangeStats>) -> Option<Html> {
    let change = change?;

    let result_label = match change.result {
        ChangeResult::Improved => "Performance has improved.",
        ChangeResult::Regressed => "Performance has regressed.",
        ChangeResult::NoChange => "No change in performance detected.",
    };

    Some(html! {
        <div class="criterion-stats-table spaced">
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
                    <tr>
                        <td>{ "Mean" }</td>
                        <td>{ format!("{:+.4}%", change.mean.lower_bound) }</td>
                        <td>{ format!("{:+.4}%", change.mean.estimate) }</td>
                        <td>{ format!("{:+.4}%", change.mean.upper_bound) }</td>
                    </tr>
                    <tr>
                        <td>{ "Median" }</td>
                        <td>{ format!("{:+.4}%", change.median.lower_bound) }</td>
                        <td>{ format!("{:+.4}%", change.median.estimate) }</td>
                        <td>{ format!("{:+.4}%", change.median.upper_bound) }</td>
                    </tr>
                </tbody>
            </table>
            <div class="criterion-confidence">
                { format!("{} (p = {:.2})", result_label, change.p_value) }
            </div>
        </div>
    })
}
