use crate::{
    components::{
        CriterionBenchmark, CriterionGroupChart, DashboardHeader, LatencyChart,
        LatencyPercentileChart, MetricPanel, RequestsPerSecChart,
    },
    serialzer::decode_dashboard,
    Route,
};
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
    let hash = location.hash().trim_start_matches('#');
    let hash_string = hash.to_string();

    match decode_dashboard(hash) {
        Ok(data) => {
            if data.tests.is_empty() && data.benchmarks.is_empty() {
                return html! { <Redirect<Route> to={Route::Home} /> };
            }

            let endpoint = data.tests.first().and_then(|t| {
                if t.endpoint.is_empty() {
                    None
                } else {
                    Some(t.endpoint.clone())
                }
            });

            let num_tests = data.tests.len();
            let num_benchmarks = data.benchmarks.len();

            html! {
                <div class="dashboard">
                    <DashboardHeader
                        description={data.description.clone()}
                        hash={hash_string}
                        endpoint={endpoint}
                        tags={data.tags}
                        tests={num_tests}
                        benchmarks={num_benchmarks}
                    />
                    if !data.tests.is_empty() {
                        <div class="dashboard-grid">
                            { for data.tests.iter().map(|test| html! {
                                <>
                                    <MetricPanel class="panel-requests-per-sec" value={ format_requests_float(test.requests_per_sec) } label="Requests per second" />
                                    <MetricPanel class="panel-total-requests" value={ format_requests(test.total_requests) } label="Total requests" />
                                    <MetricPanel class="panel-data-transferred" value={ test.transfer_per_sec.clone() } label="Data transferred" />
                                    <MetricPanel class="panel-threads" value={ test.threads.to_string() } label="Threads" />
                                    <MetricPanel class="panel-connections" value={ test.connections.to_string() } label="Connections" />
                                    <RequestsPerSecChart avg={test.req.avg} stddev={test.req.stddev} max={test.req.max} stddev_percent={test.req.stddev_percent} />
                                    <LatencyChart avg={test.latency.avg} stddev={test.latency.stddev} max={test.latency.max} stddev_percent={test.latency.stddev_percent} distribution={test.latency_distribution.clone()} />
                                    if !test.percentiles.is_empty() {
                                        <LatencyPercentileChart requests_per_sec={test.requests_per_sec} percentiles={test.percentiles.clone()} />
                                    }
                                </>
                            }) }
                        </div>
                    }
                    if !data.benchmarks.is_empty() {
                        <div class="criterion-section">
                            <CriterionGroupChart benchmarks={data.benchmarks.clone()} />
                            { for data.benchmarks.iter().map(|bench| html! {
                                <CriterionBenchmark metrics={bench.clone()} />
                            }) }
                        </div>
                    }
                    if !data.benchmarks.is_empty() {
                        <div class="criterion-footer">
                            <a href="https://bheisler.github.io/criterion.rs/book/user_guide/plots_and_graphs.html"
                               target="_blank"
                               rel="noopener noreferrer">
                                { "Understanding Criterion.rs charts →" }
                            </a>
                        </div>
                    }
                </div>
            }
        }
        Err(e) => {
            log::error!("Failed to decode dashboard data: {}", e);
            html! { <Redirect<Route> to={Route::Home} /> }
        }
    }
}
