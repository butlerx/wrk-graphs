use super::line_chart::{LineCurveChart, LineCurveChartConfig, LineCurveChartProps, Series};
use crate::parser::PercentileBucket;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub requests_per_sec: f64,
    pub percentiles: Vec<PercentileBucket>,
}

#[function_component(LatencyPercentileChart)]
pub fn latency_percentile_chart(props: &ChartProps) -> Html {
    let ChartProps {
        requests_per_sec,
        percentiles,
    } = &props;

    // check if the values should be displayed in s, ms or μs default is ms
    let (y_axis_title, scale) = if percentiles.iter().any(|p| p.value > 1000.0) {
        ("Latency (s)", 1000.0)
    } else if percentiles.iter().any(|p| p.value > 1.0) {
        ("Latency (ms)", 1.0)
    } else {
        ("Latency (μs)", 0.001)
    };

    let data_points: Vec<(f64, f64)> = percentiles
        .iter()
        .map(|p| (p.percentile * 100.0, p.value / scale))
        .collect();

    let x_labels: Vec<String> = (0..=10)
        .map(|i| format!("{:.0}%", f64::from(i) * 10.0))
        .collect();

    let chart_props = LineCurveChartProps {
        data: vec![(
            Series {
                name: format!("{requests_per_sec} req/s"),
                color: "#4a90e2".to_string(),
            },
            data_points,
        )],
        x: x_labels,
        config: LineCurveChartConfig {
            show_inflection_points: false,
            stroke_width: 2,
            show_area_chart: true,
            x_axis_title: "Percentile".to_string(),
            y_axis_title: y_axis_title.to_string(),
        },
    };

    html! {
        <div class="metric-panel panel-percentiles full-width">
            <h3>{ "Latency Percentiles" }</h3>
            <div class="percentile-chart">
                <LineCurveChart ..chart_props />
            </div>
        </div>
    }
}
