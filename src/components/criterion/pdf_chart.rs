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

const PRIMARY_COLOR: &str = "rgb(31, 120, 180)";
const AREA_COLOR: &str = "rgba(31, 120, 180, 0.2)";

struct KdePlotArea {
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
    y_max: f64,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct CriterionPdfChartProps {
    pub iteration_count: Vec<f64>,
    pub measured_values: Vec<f64>,
}

#[function_component]
pub fn CriterionPdfChart(props: &CriterionPdfChartProps) -> Html {
    let props_clone = props.clone();
    let canvas_ref = use_canvas(move |ctx, w, h| {
        draw_pdf_chart(ctx, w, h, &props_clone);
    });

    html! {
        <div style="position: relative">
            <canvas
                ref={canvas_ref}
                role="img"
                aria-label="Probability density function chart"
                style="width: 100%; height: 100%; box-sizing: border-box"
            />
        </div>
    }
}

fn draw_pdf_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &CriterionPdfChartProps,
) {
    let m = ChartMargins::default();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    let per_iter_ms = compute_per_iteration_ms(&props.iteration_count, &props.measured_values);

    draw_axes(context, width, height, &m);

    if per_iter_ms.len() < 2 {
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

    let kde_points = compute_kde(&per_iter_ms, 200);
    if kde_points.is_empty() {
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

    let x_min = kde_points.first().map_or(0.0, |(x, _)| *x);
    let x_max = kde_points.last().map_or(1.0, |(x, _)| *x);
    let max_density = kde_points
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON)
        * 1.1;
    let mean = per_iter_ms.iter().sum::<f64>() / per_iter_ms.len() as f64;

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

    draw_kde_area(context, &area, &kde_points, &m);
    draw_kde_line(context, &area, &kde_points, &m);
    draw_mean_line(context, &area, mean, &m);
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
    m: &ChartMargins,
) {
    context.set_stroke_style_str(PRIMARY_COLOR);
    context.set_line_width(2.0);
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
}

fn draw_kde_area(
    context: &CanvasRenderingContext2d,
    area: &KdePlotArea,
    points: &[(f64, f64)],
    m: &ChartMargins,
) {
    context.set_fill_style_str(AREA_COLOR);
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
    m: &ChartMargins,
) {
    let x = map_x(mean, area.x_min, area.x_max, area.width, m);
    context.set_stroke_style_str(PRIMARY_COLOR);
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(x, m.top);
    context.line_to(x, area.height - m.bottom);
    context.stroke();
}
