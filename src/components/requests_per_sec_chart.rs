use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub stddev: f64,
    pub avg: f64,
    pub max: f64,
    pub stddev_percent: f64,
}

#[function_component(RequestsPerSecChart)]
pub fn requests_per_sec_chart(props: &ChartProps) -> Html {
    html! {
        <div class="metric-panel panel-requests-per-sec-stats">
            <h3>{ "Requests per Second" }</h3>
            <div class="metric-content">
                <MetricRow label="Average" value={props.avg} />
                <MetricRow label="Standard Deviation" value={props.stddev} />
                <MetricRow label="Max" value={props.max} />
                <div class="metric-row">
                    <div class="metric-label">{ "Standard Deviation Percent" }</div>
                    <div class="metric-value">{ format!("{:.2}%",props.stddev_percent) }</div>
                </div>
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
