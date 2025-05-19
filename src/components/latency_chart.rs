use crate::parser::WrkMetrics;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub metrics: WrkMetrics,
}

#[function_component(LatencyChart)]
pub fn latency_chart(props: &ChartProps) -> Html {
    let metrics = &props.metrics;

    let mut latency_distribution: Vec<_> = metrics.latency_distribution.iter().collect();
    latency_distribution.sort_by_key(|&(k, _)| k);

    html! {
        <div class="metric-panel panel-latency-stats">
            <h3>{ "Latency" }</h3>
            <div class="metric-content">
                <MetricRow label="Average" value={metrics.latency.avg} />
                <MetricRow label="Standard Deviation" value={metrics.latency.stdev} />
                <MetricRow label="Max" value={metrics.latency.max} />
            </div>
            if !latency_distribution.is_empty() {
                <h4>{ "Latency Distribution" }</h4>
                <div class="metric-content">
                    { for latency_distribution.into_iter().map(|(key, value)| html! { <MetricRow label={key.clone()} value={*value} /> }) }
                </div>
            }
        </div>
    }
}

fn format_latency(value: f64) -> String {
    if value < 1.0 {
        format!("{:.2}us", value * 1000.0)
    } else {
        format!("{:.2}ms", value)
    }
}

#[derive(Properties, PartialEq)]
struct MetricRowProps {
    pub label: String,
    pub value: f64,
}

#[function_component(MetricRow)]
fn metric_row(props: &MetricRowProps) -> Html {
    let MetricRowProps { value, label } = props;
    html! {
        <div class="metric-row">
            <div class="metric-label">{ label }</div>
            <div class="metric-value">{ format_latency(*value) }</div>
        </div>
    }
}
