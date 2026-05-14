#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::components::charts::chart_utils::setup_canvas;
use crate::parser::criterion::CriterionMetrics;
use gloo::events::EventListener;
use std::collections::BTreeMap;
use web_sys::{window, CanvasRenderingContext2d};
use yew::prelude::*;

const PALETTE: [&str; 6] = [
    "#1f78b4", "#33a02c", "#e31a1c", "#ff7f00", "#6a3d9a", "#b15928",
];

#[derive(Clone, Debug, PartialEq)]
struct ChartSeries {
    name: String,
    color: String,
    points: Vec<(f64, f64)>,
}

#[derive(Properties, PartialEq)]
pub struct CriterionLineChartProps {
    pub benchmarks: Vec<CriterionMetrics>,
}

#[function_component(CriterionLineChart)]
pub fn criterion_line_chart(props: &CriterionLineChartProps) -> Html {
    let canvas_ref = use_node_ref();
    let series = build_series(&props.benchmarks);

    {
        let canvas_ref = canvas_ref.clone();
        let series_clone = series.clone();
        use_effect_with((), move |()| {
            let resize_callback = {
                let canvas_ref = canvas_ref.clone();
                move || {
                    if let Some((ctx, w, h)) = setup_canvas(&canvas_ref) {
                        draw_chart(&ctx, w, h, &series_clone);
                    }
                }
            };

            resize_callback();

            let listener = window().map(|win| {
                EventListener::new(&win, "resize", move |_event| {
                    resize_callback();
                })
            });

            move || drop(listener)
        });
    }

    html! {
        <div style="position: relative">
            <canvas
                ref={canvas_ref}
                role="img"
                aria-label="Criterion benchmark line chart"
                style="width: 100%; height: 100%; box-sizing: border-box"
            />
            <div
                style="
                    position: absolute;
                    top: 10px;
                    right: 10px;
                    background-color: rgba(255, 255, 255, 0.9);
                    padding: 6px 8px;
                    border-radius: 4px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1)
                "
            >
                <div style="font-size: 11px; font-weight: 600; margin-bottom: 4px">
                    { "Legend" }
                </div>
                <div style="display: flex; flex-direction: column; gap: 4px">
                    { for series.iter().map(|s| {
                            html! {
                                <div style="display: flex; flex-direction: row; align-items: center; gap: 6px; font-size: 11px;">
                                    <div style={format!("background-color: {}; width: 10px; height: 10px;", s.color)}></div>
                                    <span>{ &s.name }</span>
                                </div>
                            }
                        }) }
                </div>
            </div>
        </div>
    }
}

fn build_series(benchmarks: &[CriterionMetrics]) -> Vec<ChartSeries> {
    let mut grouped: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();

    for benchmark in benchmarks {
        if let Some((function_name, input_size)) = split_benchmark_name(&benchmark.name) {
            let mean_ms = benchmark
                .mean
                .as_ref()
                .map_or(benchmark.time.estimate, |m| m.estimate);
            grouped
                .entry(function_name)
                .or_default()
                .push((input_size, mean_ms));
        }
    }

    grouped
        .into_iter()
        .enumerate()
        .map(|(idx, (name, mut points))| {
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            ChartSeries {
                name,
                color: PALETTE[idx % PALETTE.len()].to_string(),
                points,
            }
        })
        .collect()
}

fn split_benchmark_name(name: &str) -> Option<(String, f64)> {
    let (prefix, last) = name.rsplit_once('/')?;
    let numeric: f64 = last.parse().ok()?;
    Some((prefix.to_string(), numeric))
}

fn draw_chart(context: &CanvasRenderingContext2d, width: f64, height: f64, series: &[ChartSeries]) {
    context.clear_rect(0.0, 0.0, width, height);
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    let axis_left = 60.0;
    let axis_right = 24.0;
    let axis_top = 20.0;
    let axis_bottom = 56.0;

    if series.is_empty() {
        context.set_fill_style_str("#777");
        context.set_font("12px monospace");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        let _ = context.fill_text("No comparable benchmark points", width / 2.0, height / 2.0);
        return;
    }

    let x_min = series
        .iter()
        .flat_map(|s| s.points.iter().map(|(x, _)| *x))
        .fold(f64::INFINITY, f64::min);
    let x_max = series
        .iter()
        .flat_map(|s| s.points.iter().map(|(x, _)| *x))
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = 0.0;
    let y_data_max = series
        .iter()
        .flat_map(|s| s.points.iter().map(|(_, y)| *y))
        .fold(f64::NEG_INFINITY, f64::max);
    let y_max = if y_data_max <= 0.0 {
        1.0
    } else {
        y_data_max * 1.1
    };

    let plot_width = width - axis_left - axis_right;
    let plot_height = height - axis_top - axis_bottom;

    let x_to_px = |x: f64| {
        if (x_max - x_min).abs() < f64::EPSILON {
            axis_left + plot_width / 2.0
        } else {
            axis_left + ((x - x_min) / (x_max - x_min)) * plot_width
        }
    };
    let y_to_px = |y: f64| axis_top + (1.0 - ((y - y_min) / (y_max - y_min))) * plot_height;

    draw_grid_and_axes(
        context,
        width,
        height,
        axis_left,
        axis_right,
        axis_top,
        axis_bottom,
        x_min,
        x_max,
        y_min,
        y_max,
        x_to_px,
        y_to_px,
    );

    for line in series {
        context.begin_path();
        context.set_stroke_style_str(&line.color);
        context.set_line_width(2.0);

        for (idx, (x, y)) in line.points.iter().enumerate() {
            let px = x_to_px(*x);
            let py = y_to_px(*y);
            if idx == 0 {
                context.move_to(px, py);
            } else {
                context.line_to(px, py);
            }
        }
        context.stroke();

        context.set_fill_style_str(&line.color);
        for (x, y) in &line.points {
            let px = x_to_px(*x);
            let py = y_to_px(*y);
            context.begin_path();
            let _ = context.arc(px, py, 3.0, 0.0, std::f64::consts::PI * 2.0);
            context.fill();
        }
    }

    context.set_fill_style_str("#111");
    context.set_font("bold 12px monospace");
    context.set_text_align("center");
    context.set_text_baseline("middle");
    let _ = context.fill_text("Input Size", width / 2.0, height - 18.0);

    context.save();
    let _ = context.rotate(-std::f64::consts::PI / 2.0);
    let _ = context.fill_text("Mean Time (ms)", -(height / 2.0), 16.0);
    context.restore();
}

#[allow(clippy::too_many_arguments)]
fn draw_grid_and_axes(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    axis_left: f64,
    axis_right: f64,
    axis_top: f64,
    axis_bottom: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    x_to_px: impl Fn(f64) -> f64,
    y_to_px: impl Fn(f64) -> f64,
) {
    let x_grid_lines = 6;
    let y_grid_lines = 5;

    context.set_stroke_style_str("#e0e0e0");
    context.set_line_width(1.0);
    context.set_font("10px monospace");
    context.set_fill_style_str("#444");

    for i in 0..=x_grid_lines {
        let ratio = f64::from(i) / f64::from(x_grid_lines);
        let value = x_min + ratio * (x_max - x_min);
        let x = x_to_px(value);

        context.begin_path();
        context.move_to(x, axis_top);
        context.line_to(x, height - axis_bottom);
        context.stroke();

        context.set_text_align("center");
        context.set_text_baseline("top");
        let _ = context.fill_text(&format_number(value), x, height - axis_bottom + 6.0);
    }

    for i in 0..=y_grid_lines {
        let ratio = f64::from(i) / f64::from(y_grid_lines);
        let value = y_min + ratio * (y_max - y_min);
        let y = y_to_px(value);

        context.begin_path();
        context.move_to(axis_left, y);
        context.line_to(width - axis_right, y);
        context.stroke();

        context.set_text_align("right");
        context.set_text_baseline("middle");
        let _ = context.fill_text(&format_ms(value), axis_left - 8.0, y);
    }

    context.set_stroke_style_str("#bdbdbd");
    context.set_line_width(1.5);
    context.begin_path();
    context.move_to(axis_left, axis_top);
    context.line_to(axis_left, height - axis_bottom);
    context.line_to(width - axis_right, height - axis_bottom);
    context.stroke();
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON || value.abs() >= 1000.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn format_ms(value: f64) -> String {
    if value < 0.001 {
        format!("{:.2}ns", value * 1_000_000.0)
    } else if value < 1.0 {
        format!("{:.2}µs", value * 1000.0)
    } else if value >= 1000.0 {
        format!("{:.2}s", value / 1000.0)
    } else {
        format!("{value:.3}ms")
    }
}
