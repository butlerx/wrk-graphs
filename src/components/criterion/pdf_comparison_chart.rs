#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
use gloo::events::EventListener;
use web_sys::{wasm_bindgen::JsCast, window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use crate::components::charts::chart_utils::{
    draw_axes, draw_axis_titles, draw_x_grid_and_labels, draw_y_grid_and_labels, format_tick_value,
    map_x, map_y, ChartMargins, GridConfig,
};

const BASELINE_COLOR: &str = "rgb(228, 26, 28)";
const BASELINE_AREA_COLOR: &str = "rgba(228, 26, 28, 0.15)";
const CURRENT_COLOR: &str = "rgb(31, 120, 180)";
const CURRENT_AREA_COLOR: &str = "rgba(31, 120, 180, 0.15)";

struct KdePlotArea {
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
    y_max: f64,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct CriterionPdfComparisonChartProps {
    pub iteration_count: Vec<f64>,
    pub measured_values: Vec<f64>,
    pub baseline_iteration_count: Vec<f64>,
    pub baseline_measured_values: Vec<f64>,
}

#[function_component]
pub fn CriterionPdfComparisonChart(props: &CriterionPdfComparisonChartProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let props_clone = props.clone();
        use_effect_with((), move |()| {
            let canvas = canvas_ref
                .cast::<HtmlCanvasElement>()
                .expect("Failed to get canvas element");

            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            let props_clone_resize = props_clone.clone();
            let resize_callback = {
                let canvas_ref = canvas_ref.clone();
                move || {
                    let canvas = canvas_ref
                        .cast::<HtmlCanvasElement>()
                        .expect("Failed to get canvas element");

                    let device_pixel_ratio = window().unwrap().device_pixel_ratio();
                    let parent = canvas.parent_element().unwrap();
                    let width = f64::from(parent.client_width());
                    let height = width * 0.6;

                    canvas.set_width((width * device_pixel_ratio) as u32);
                    canvas.set_height((height * device_pixel_ratio) as u32);

                    context
                        .scale(device_pixel_ratio, device_pixel_ratio)
                        .unwrap();

                    draw_pdf_comparison_chart(&context, width, height, &props_clone_resize);
                }
            };

            resize_callback();

            let listener = EventListener::new(&window().unwrap(), "resize", move |_event| {
                resize_callback();
            });

            move || drop(listener)
        });
    }

    html! {
        <div style="position: relative">
            <canvas ref={canvas_ref} style="width: 100%; height: 100%; box-sizing: border-box" />
        </div>
    }
}

fn draw_pdf_comparison_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &CriterionPdfComparisonChartProps,
) {
    let m = ChartMargins::default();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    let current_ms = compute_per_iteration_ms(&props.iteration_count, &props.measured_values);
    let baseline_ms = compute_per_iteration_ms(
        &props.baseline_iteration_count,
        &props.baseline_measured_values,
    );

    draw_axes(context, width, height, &m);

    let current_kde = if current_ms.len() >= 2 {
        compute_kde(&current_ms, 200)
    } else {
        Vec::new()
    };

    let baseline_kde = if baseline_ms.len() >= 2 {
        compute_kde(&baseline_ms, 200)
    } else {
        Vec::new()
    };

    if current_kde.is_empty() && baseline_kde.is_empty() {
        draw_axis_titles(
            context,
            width,
            height,
            &m,
            "Average Iteration Time (ms)",
            "Density",
        );
        return;
    }

    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut max_density: f64 = 0.0;

    if !current_kde.is_empty() {
        x_min = x_min.min(current_kde.first().unwrap().0);
        x_max = x_max.max(current_kde.last().unwrap().0);
        max_density = max_density.max(current_kde.iter().map(|(_, y)| *y).fold(0.0, f64::max));
    }

    if !baseline_kde.is_empty() {
        x_min = x_min.min(baseline_kde.first().unwrap().0);
        x_max = x_max.max(baseline_kde.last().unwrap().0);
        max_density = max_density.max(baseline_kde.iter().map(|(_, y)| *y).fold(0.0, f64::max));
    }

    max_density = max_density.max(f64::EPSILON) * 1.1;

    draw_x_grid_and_labels(
        context,
        width,
        height,
        &m,
        &GridConfig {
            min: x_min,
            max: x_max,
            num_lines: 5,
            fmt: format_tick_value,
        },
    );
    draw_y_grid_and_labels(
        context,
        width,
        height,
        &m,
        &GridConfig {
            min: 0.0,
            max: max_density,
            num_lines: 5,
            fmt: format_tick_value,
        },
    );

    let area = KdePlotArea {
        width,
        height,
        x_min,
        x_max,
        y_max: max_density,
    };

    if !baseline_kde.is_empty() {
        draw_kde_area(context, &area, &baseline_kde, BASELINE_AREA_COLOR, &m);
        draw_kde_line(context, &area, &baseline_kde, BASELINE_COLOR, false, &m);
        let mean = baseline_ms.iter().sum::<f64>() / baseline_ms.len() as f64;
        draw_mean_line(context, &area, mean, BASELINE_COLOR, true, &m);
    }

    if !current_kde.is_empty() {
        draw_kde_area(context, &area, &current_kde, CURRENT_AREA_COLOR, &m);
        draw_kde_line(context, &area, &current_kde, CURRENT_COLOR, false, &m);
        let mean = current_ms.iter().sum::<f64>() / current_ms.len() as f64;
        draw_mean_line(context, &area, mean, CURRENT_COLOR, false, &m);
    }

    draw_legend(context, width, &m);
    draw_axis_titles(
        context,
        width,
        height,
        &m,
        "Average Iteration Time (ms)",
        "Density",
    );
}

fn compute_per_iteration_ms(iteration_count: &[f64], measured_values: &[f64]) -> Vec<f64> {
    iteration_count
        .iter()
        .zip(measured_values.iter())
        .filter_map(|(iters, measured_ns)| {
            if *iters > 0.0 && iters.is_finite() && measured_ns.is_finite() {
                Some((measured_ns / iters) / 1_000_000.0)
            } else {
                None
            }
        })
        .collect()
}

fn compute_kde(values: &[f64], points: usize) -> Vec<(f64, f64)> {
    if values.is_empty() || points < 2 {
        return Vec::new();
    }

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt();

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }

    let x_min = if (max - min).abs() < f64::EPSILON {
        min - 1.0
    } else {
        min
    };
    let x_max = if (max - min).abs() < f64::EPSILON {
        max + 1.0
    } else {
        max
    };

    let silverman = 1.06 * sigma * n.powf(-0.2);
    let h = if silverman.is_finite() && silverman > 0.0 {
        silverman
    } else {
        ((x_max - x_min).abs() / 20.0).max(1e-9)
    };

    let step = (x_max - x_min) / (points as f64 - 1.0);
    let norm = 1.0 / ((2.0 * std::f64::consts::PI).sqrt() * h * n);

    (0..points)
        .map(|i| {
            let x = x_min + i as f64 * step;
            let sum = values
                .iter()
                .map(|v| {
                    let u = (x - v) / h;
                    (-0.5 * u * u).exp()
                })
                .sum::<f64>();
            (x, norm * sum)
        })
        .collect()
}

fn draw_kde_line(
    context: &CanvasRenderingContext2d,
    area: &KdePlotArea,
    points: &[(f64, f64)],
    color: &str,
    dashed: bool,
    m: &ChartMargins,
) {
    context.set_stroke_style_str(color);
    context.set_line_width(2.0);
    if dashed {
        let dash_array = js_sys::Array::new();
        dash_array.push(&wasm_bindgen::JsValue::from_f64(5.0));
        dash_array.push(&wasm_bindgen::JsValue::from_f64(5.0));
        context.set_line_dash(&dash_array).unwrap();
    } else {
        context.set_line_dash(&js_sys::Array::new()).unwrap();
    }
    context.begin_path();

    for (idx, (x, y)) in points.iter().enumerate() {
        let px = map_x(*x, area.x_min, area.x_max, area.width, m);
        let py = map_y(*y, 0.0, area.y_max, area.height, m);
        if idx == 0 {
            context.move_to(px, py);
        } else {
            context.line_to(px, py);
        }
    }

    context.stroke();
    context.set_line_dash(&js_sys::Array::new()).unwrap();
}

fn draw_kde_area(
    context: &CanvasRenderingContext2d,
    area: &KdePlotArea,
    points: &[(f64, f64)],
    color: &str,
    m: &ChartMargins,
) {
    context.set_fill_style_str(color);
    context.begin_path();

    for (idx, (x, y)) in points.iter().enumerate() {
        let px = map_x(*x, area.x_min, area.x_max, area.width, m);
        let py = map_y(*y, 0.0, area.y_max, area.height, m);
        if idx == 0 {
            context.move_to(px, py);
        } else {
            context.line_to(px, py);
        }
    }

    if let Some((last_x, _)) = points.last() {
        context.line_to(
            map_x(*last_x, area.x_min, area.x_max, area.width, m),
            area.height - m.bottom,
        );
    }
    if let Some((first_x, _)) = points.first() {
        context.line_to(
            map_x(*first_x, area.x_min, area.x_max, area.width, m),
            area.height - m.bottom,
        );
    }
    context.close_path();
    context.fill();
}

fn draw_mean_line(
    context: &CanvasRenderingContext2d,
    area: &KdePlotArea,
    mean: f64,
    color: &str,
    dashed: bool,
    m: &ChartMargins,
) {
    let x = map_x(mean, area.x_min, area.x_max, area.width, m);
    context.set_stroke_style_str(color);
    context.set_line_width(2.0);
    if dashed {
        let dash_array = js_sys::Array::new();
        dash_array.push(&wasm_bindgen::JsValue::from_f64(5.0));
        dash_array.push(&wasm_bindgen::JsValue::from_f64(5.0));
        context.set_line_dash(&dash_array).unwrap();
    } else {
        context.set_line_dash(&js_sys::Array::new()).unwrap();
    }
    context.begin_path();
    context.move_to(x, m.top);
    context.line_to(x, area.height - m.bottom);
    context.stroke();
    context.set_line_dash(&js_sys::Array::new()).unwrap();
}

fn draw_legend(context: &CanvasRenderingContext2d, width: f64, m: &ChartMargins) {
    let legend_x = width - m.right - 100.0;
    let legend_y = m.top + 10.0;

    context.set_font("10px monospace");
    context.set_text_align("left");
    context.set_text_baseline("middle");

    context.set_fill_style_str(BASELINE_COLOR);
    context.fill_rect(legend_x, legend_y, 10.0, 10.0);
    context.set_fill_style_str("black");
    context
        .fill_text("Previous", legend_x + 15.0, legend_y + 5.0)
        .unwrap();

    context.set_fill_style_str(CURRENT_COLOR);
    context.fill_rect(legend_x, legend_y + 20.0, 10.0, 10.0);
    context.set_fill_style_str("black");
    context
        .fill_text("Current", legend_x + 15.0, legend_y + 25.0)
        .unwrap();
}
