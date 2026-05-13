use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MetricPanelProps {
    pub label: String,
    pub value: String,
    pub class: Option<String>,
}

#[function_component(MetricPanel)]
pub fn metric_panel(props: &MetricPanelProps) -> Html {
    let MetricPanelProps {
        label,
        value,
        class,
    } = props;
    let class = if let Some(class_name) = class {
        format!("metric-panel {class_name}")
    } else {
        "metric-panel".to_string()
    };
    html! {
        <div class={class}>
            <div class="metric-content">
                <div class="main-value">{ value }</div>
                <div class="metric-label">{ label }</div>
            </div>
        </div>
    }
}
