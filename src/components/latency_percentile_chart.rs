use crate::parser::WrkMetrics;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub metrics: WrkMetrics,
}

fn format_latency(value: f64) -> String {
    if value < 1.0 {
        format!("{:.2}us", value * 1000.0)
    } else {
        format!("{value:.2}ms")
    }
}

#[function_component(LatencyPercentileChart)]
pub fn latency_percentile_chart(props: &ChartProps) -> Html {
    let metrics = &props.metrics;
    let percentiles = &metrics.percentile_spectrum.percentiles;

    // Calculate max value for y-axis scaling
    let max_value = percentiles.iter().map(|p| p.value).fold(0.0, f64::max);
    let y_scale = (max_value * 1.2).ceil(); // Add 20% padding and round up

    html! {
        <div class="metric-panel panel-percentiles full-width">
            <h3>{ "Latency Percentiles" }</h3>
            <div class="metric-content">
                <div class="percentile-chart">
                    <svg viewBox="0 0 800 400" class="chart-svg">
                        // Main axes
                        <line
                            x1="50"
                            y1="350"
                            x2="750"
                            y2="350"
                            class="axis-line"
                            stroke-width="3"
                            stroke="#ffffff"
                        />
                        <line
                            x1="50"
                            y1="350"
                            x2="50"
                            y2="50"
                            class="axis-line"
                            stroke-width="3"
                            stroke="#ffffff"
                        />
                        // Grid lines
                        { for (0..=4).map(|i| {
                            let y = 50.0 + (f64::from(i) * 75.0);
                            html! {
                                <>
                                    <line
                                        x1="50"
                                        y1={y.to_string()}
                                        x2="750"
                                        y2={y.to_string()}
                                        class="grid-line"
                                        stroke="#444444"
                                    />
                                    <text
                                        x="35"
                                        y={y.to_string()}
                                        class="axis-label"
                                        text-anchor="end"
                                        fill="white"
                                        font-size="10"
                                    >
                                        { format_latency((4.0 - f64::from(i)) * y_scale / 4.0) }
                                    </text>
                                </>
                            }
                        }) }
                        { for (0..=4).map(|i| {
                            let x = 50.0 + (f64::from(i) * 175.0);
                            html! {
                                <>
                                    <line
                                        x1={x.to_string()}
                                        y1="50"
                                        x2={x.to_string()}
                                        y2="350"
                                        class="grid-line"
                                        stroke="#444444"
                                    />
                                    <text
                                        x={x.to_string()}
                                        y="370"
                                        class="axis-label"
                                        text-anchor="middle"
                                        fill="white"
                                        font-size="10"
                                    >
                                        { format!("{}%", i * 25) }
                                    </text>
                                </>
                            }
                        }) }
                        // Data points and line
                        { for percentiles.iter().map(|p| {
                            let x = 50.0 + (p.percentile * 700.0);
                            let y = 350.0 - (p.value / y_scale * 300.0);
                            html! {
                                <circle
                                    cx={x.to_string()}
                                    cy={y.to_string()}
                                    r="3"
                                    class="percentile-point"
                                    title={format!("{:.1}%: {}",  p.percentile * 100.0, format_latency(p.value))}
                                />
                            }
                        }) }
                        <path
                            d={format!(
                                "M {} {}",
                                percentiles.first().map(|p| {
                                    let x = 50.0 + (p.percentile * 700.0);
                                    let y = 350.0 - (p.value / y_scale * 300.0);
                                    format!("{x} {y}")
                                }).unwrap_or_default(),
                                percentiles.iter().skip(1).map(|p| {
                                    let x = 50.0 + (p.percentile * 700.0);
                                    let y = 350.0 - (p.value / y_scale * 300.0);
                                    format!("L {x} {y}")
                                }).collect::<Vec<_>>().join(" ")
                            )}
                            class="percentile-line"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        />
                    </svg>
                </div>
            </div>
        </div>
    }
}
