use crate::{
    components::{LatencyChart, LatencyPercentileChart, RequestsPerSecChart},
    serialzer::decode_dashboard,
    Route,
};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

fn format_requests(value: u64) -> String {
    match value {
        v if v >= 1_000_000 => format!("{:.2}M", v / 1_000_000),
        v if v >= 1_000 => format!("{:.2}k", v / 1_000),
        _ => format!("{value:.2}"),
    }
}

fn format_requests_float(value: f64) -> String {
    match value {
        v if v >= 1_000_000.0 => format!("{:.2}M", v / 1_000_000.0),
        v if v >= 1_000.0 => format!("{:.2}k", v / 1_000.0),
        _ => format!("{value:.2}"),
    }
}

#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {
    let location = use_location().unwrap();
    let navigator = use_navigator().unwrap();
    let hash = location.hash().trim_start_matches('#');
    let copied = use_state(|| false);
    let copied_embed = use_state(|| false);

    let on_header_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&crate::Route::Home);
        })
    };

    let window = web_sys::window().expect("window will exist");
    let url = format!("{}/dashboard#{}", window.location().origin().unwrap(), hash);
    let embed_code = format!(
        "<iframe src=\"{url}\" width=\"100%\" height=\"600px\" frameborder=\"0\"></iframe>"
    );

    let on_copy = {
        let window = window.clone();
        let copied = copied.clone();
        let url = url.clone();
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

    let on_copy_embed = {
        let window = window.clone();
        let copied_embed = copied_embed.clone();
        let embed_code = embed_code.clone();
        Callback::from(move |_| {
            let _ = window.navigator().clipboard().write_text(&embed_code);

            copied_embed.set(true);

            let window = window.clone();
            let copied_embed = copied_embed.clone();
            let closure = Closure::once(move || {
                copied_embed.set(false);
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                2000,
            );
            closure.forget();
        })
    };

    match decode_dashboard(hash) {
        Ok(data) => {
            let metrics = &data.metrics;
            html! {
                <div class="dashboard">
                    <header class="dashboard-header">
                        <div class="header-content">
                            <div class="header-left" onclick={on_header_click}>
                                <img src="./icon.png" alt="Logo" class="header-icon" />
                                <h1>{ "Load Test Results" }</h1>
                            </div>
                            <div class="share-buttons">
                                <button onclick={on_copy} class="share-button">
                                    { if *copied { "Copied!" } else { "Copy URL" } }
                                </button>
                                <button onclick={on_copy_embed} class="share-button">
                                    { if *copied_embed { "Copied!" } else { "Copy Embed Code" } }
                                </button>
                            </div>
                        </div>
                        <div class="metadata">
                            if let Some(description) = data.description {
                                <div class="metadata-row">
                                    <span class="metadata-label">{ "Description:" }</span>
                                    <span class="metadata-value">{ description }</span>
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
                        <RequestsPerSecChart
                            avg={metrics.req.avg}
                            stddev={metrics.req.stddev}
                            max={metrics.req.max}
                            stddev_percent={metrics.req.stddev_percent}
                        />
                        <LatencyChart
                            avg={metrics.latency.avg}
                            stddev={metrics.latency.stddev}
                            max={metrics.latency.max}
                            stddev_percent={metrics.latency.stddev_percent}
                            distribution={metrics.latency_distribution.clone()}
                        />
                        if !metrics.percentiles.is_empty() {
                            <LatencyPercentileChart
                                requests_per_sec={metrics.requests_per_sec}
                                percentiles={metrics.percentiles.clone()}
                            />
                        }
                    </div>
                </div>
            }
        }
        Err(e) => {
            log::error!("Failed to decode dashboard data: {}", e);
            html! { <Redirect<Route> to={Route::Home} /> }
        }
    }
}
