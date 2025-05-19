use crate::parser::WrkMetrics;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub metrics: WrkMetrics,
}

#[function_component(RequestsPerSecChart)]
pub fn requests_per_sec_chart(props: &ChartProps) -> Html {
    let metrics = &props.metrics;

    html! {
        <div class="metric-panel panel-requests-per-sec-stats">
            <h3>{ "Requests per Second" }</h3>
            <div class="metric-content">
                <MetricRow label="Average" value={metrics.req.avg} />
                <MetricRow label="Standard Deviation" value={metrics.req.stdev} />
                <MetricRow label="Max" value={metrics.req.max} />
            </div>
        </div>
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
    let formatted_value = if *value >= 1000.0 {
        format!("{:.2}k", value / 1000.0)
    } else {
        format!("{value:.2}")
    };
    html! {
        <div class="metric-row">
            <div class="metric-label">{ label }</div>
            <div class="metric-value">{ formatted_value }</div>
        </div>
    }
}
