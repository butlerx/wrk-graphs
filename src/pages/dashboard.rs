use crate::{
    components::{LatencyChart, LatencyPercentileChart, RequestsPerSecChart},
    pages::NotFoundPage,
    parser,
};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Loadtest {
    pub metrics: parser::WrkMetrics,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Properties, PartialEq)]
pub struct DashboardProps {
    pub hash: String,
}

fn decode_dashboard(hash: &str) -> Option<Loadtest> {
    let data = BASE64_URL_SAFE_NO_PAD.decode(hash).ok()?;
    let data_str = String::from_utf8(data).ok()?;
    serde_json5::from_str::<Loadtest>(&data_str).ok()
}

fn format_requests(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.2}M", value / 1_000_000)
    } else if value >= 1_000 {
        format!("{:.2}k", value / 1_000)
    } else {
        format!("{value:.2}")
    }
}

fn format_requests_float(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}k", value / 1_000.0)
    } else {
        format!("{value:.2}")
    }
}

#[function_component(DashboardPage)]
pub fn dashboard_page(props: &DashboardProps) -> Html {
    let hash = props.hash.clone();
    let copied = use_state(|| false);

    let window = web_sys::window().expect("window will exist");
    let url = format!(
        "{}/dashboard/{}",
        window.location().origin().unwrap(),
        hash.clone()
    );

    let on_copy = {
        let copied = copied.clone();
        Callback::from(move |_| {
            let _ = window.navigator().clipboard().write_text(&url);

            copied.set(true);

            let window = window.clone();
            let copied = copied.clone();
            let closure = Closure::once(move || {
                copied.set(false);
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                2000,
            );
            closure.forget();
        })
    };

    match decode_dashboard(&hash) {
        Some(data) => {
            let metrics = &data.metrics;
            html! {
                <div class="dashboard">
                    <header class="dashboard-header">
                        <h1>{ "Load Test Results" }</h1>
                        <div class="metadata">
                            if !data.description.is_empty() {
                                <div class="metadata-row">
                                    <span class="metadata-label">{ "Description:" }</span>
                                    <span class="metadata-value">{ data.description }</span>
                                </div>
                            }
                            if !metrics.endpoint.is_empty() {
                                <div class="metadata-row">
                                    <span class="metadata-label">{ "Endpoint:" }</span>
                                    <span class="metadata-value">{ metrics.endpoint.clone() }</span>
                                </div>
                            }
                            if !data.tags.is_empty() {
                                <div class="metadata-row">
                                    <span class="metadata-label">{ "Tags:" }</span>
                                    <div class="tag-list">
                                        { for data.tags.iter().map(|tag| html! { <span class="tag">{tag}</span> }) }
                                    </div>
                                </div>
                            }
                        </div>
                    </header>
                    <div class="dashboard-grid">
                        // Requests per second panel
                        <div class="metric-panel panel-requests-per-sec">
                            <div class="metric-content">
                                <div class="main-value">
                                    { format_requests_float(metrics.requests_per_sec) }
                                </div>
                                <div class="metric-label">{ "Requests per second" }</div>
                            </div>
                        </div>
                        // Total requests panel
                        <div class="metric-panel panel-total-requests">
                            <div class="metric-content">
                                <div class="main-value">
                                    { format_requests(metrics.total_requests) }
                                </div>
                                <div class="metric-label">{ "Total requests" }</div>
                            </div>
                        </div>
                        // Data transferred panel
                        <div class="metric-panel panel-data-transferred">
                            <div class="metric-content">
                                <div class="main-value">
                                    { metrics.transfer_per_sec.to_string() }
                                </div>
                                <div class="metric-label">{ "Data transferred" }</div>
                            </div>
                        </div>
                        // Threads panel
                        <div class="metric-panel panel-threads">
                            <div class="metric-content">
                                <div class="main-value">{ format!("{}", metrics.threads) }</div>
                                <div class="metric-label">{ "Threads" }</div>
                            </div>
                        </div>
                        // Connections panel
                        <div class="metric-panel panel-connections">
                            <div class="metric-content">
                                <div class="main-value">{ format!("{}", metrics.connections) }</div>
                                <div class="metric-label">{ "Connections" }</div>
                            </div>
                        </div>
                        <RequestsPerSecChart metrics={metrics.clone()} />
                        <LatencyChart metrics={metrics.clone()} />
                        if !metrics.percentile_spectrum.percentiles.is_empty() {
                            <LatencyPercentileChart metrics={metrics.clone()} />
                        }
                    </div>
                    <div class="share-link">
                        <h3>{ "Share this result" }</h3>
                        <p>{ "Copy this URL to share these results:" }</p>
                        <div class="copy-container">
                            <input
                                type="text"
                                readonly=true
                                value={format!("{}/dashboard/{}", web_sys::window().expect("window will exist").location().origin().unwrap(), props.hash)}
                            />
                            <button onclick={on_copy} class="copy-button">
                                { if *copied { "Copied!" } else { "Copy" } }
                            </button>
                        </div>
                    </div>
                </div>
            }
        }
        None => {
            html! { <NotFoundPage /> }
        }
    }
}
