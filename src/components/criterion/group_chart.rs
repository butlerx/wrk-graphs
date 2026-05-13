use super::CriterionLineChart;
use crate::parser::criterion::CriterionMetrics;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CriterionGroupChartProps {
    pub benchmarks: Vec<CriterionMetrics>,
}

#[function_component(CriterionGroupChart)]
pub fn criterion_group_chart(props: &CriterionGroupChartProps) -> Html {
    let benchmarks = &props.benchmarks;

    if benchmarks.len() < 2 {
        return html! {};
    }

    let has_numeric_params = benchmarks.iter().any(|b| {
        b.name
            .rsplit('/')
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .is_some()
    });

    if !has_numeric_params {
        return html! {};
    }

    html! {
        <div class="criterion-group-chart">
            <h4 class="chart-title">{"Benchmark Group Comparison"}</h4>
            <CriterionLineChart benchmarks={benchmarks.clone()} />
            <p class="chart-description">{"This chart shows how the benchmark's execution time changes across different parameter values. Each point represents the estimated time for that input size."}</p>
        </div>
    }
}
