#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
use web_sys::CanvasRenderingContext2d;
use yew::prelude::*;

use crate::drawing::{
    draw_axes, draw_axis_titles, draw_x_grid_and_labels, draw_y_grid_and_labels, format_tick_value,
    map_x, map_y, ChartMargins, GridConfig,
};
use crate::hooks::use_canvas;

use super::data::{compute_kde, compute_per_iteration_ms};

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
    let props_clone = props.clone();
    let canvas_ref = use_canvas(move |ctx, w, h| {
        draw_pdf_comparison_chart(ctx, w, h, &props_clone);
    });

    html! {
        <div style="position: relative">
            <canvas
                ref={canvas_ref}
                role="img"
                aria-label="Probability density comparison chart"
                style="width: 100%; height: 100%; box-sizing: border-box"
            />
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

    if let (Some(first), Some(last)) = (current_kde.first(), current_kde.last()) {
        x_min = x_min.min(first.0);
        x_max = x_max.max(last.0);
        max_density = max_density.max(current_kde.iter().map(|(_, y)| *y).fold(0.0, f64::max));
    }

    if let (Some(first), Some(last)) = (baseline_kde.first(), baseline_kde.last()) {
        x_min = x_min.min(first.0);
        x_max = x_max.max(last.0);
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
        let _ = context.set_line_dash(&dash_array);
    } else {
        let _ = context.set_line_dash(&js_sys::Array::new());
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
    let _ = context.set_line_dash(&js_sys::Array::new());
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
        let _ = context.set_line_dash(&dash_array);
    } else {
        let _ = context.set_line_dash(&js_sys::Array::new());
    }
    context.begin_path();
    context.move_to(x, m.top);
    context.line_to(x, area.height - m.bottom);
    context.stroke();
    let _ = context.set_line_dash(&js_sys::Array::new());
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
    let _ = context.fill_text("Previous", legend_x + 15.0, legend_y + 5.0);

    context.set_fill_style_str(CURRENT_COLOR);
    context.fill_rect(legend_x, legend_y + 20.0, 10.0, 10.0);
    context.set_fill_style_str("black");
    let _ = context.fill_text("Current", legend_x + 15.0, legend_y + 25.0);
}
