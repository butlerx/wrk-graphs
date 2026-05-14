#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
use web_sys::CanvasRenderingContext2d;
use yew::prelude::*;

use crate::components::charts::chart_utils::{
    draw_axes, draw_axis_titles, draw_x_grid_and_labels, draw_y_grid_and_labels, format_tick_value,
    map_x, map_y, ChartMargins, GridConfig,
};
use crate::components::charts::use_canvas;

const PRIMARY_COLOR: &str = "rgb(31, 120, 180)";

struct PlotArea {
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct CriterionIterationTimesChartProps {
    pub iteration_count: Vec<f64>,
    pub measured_values: Vec<f64>,
}

#[function_component]
pub fn CriterionIterationTimesChart(props: &CriterionIterationTimesChartProps) -> Html {
    let props_clone = props.clone();
    let canvas_ref = use_canvas(move |ctx, w, h| {
        draw_iteration_times_chart(ctx, w, h, &props_clone);
    });

    html! {
        <div style="position: relative">
            <canvas
                ref={canvas_ref}
                role="img"
                aria-label="Iteration times chart"
                style="width: 100%; height: 100%; box-sizing: border-box"
            />
        </div>
    }
}

fn draw_iteration_times_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &CriterionIterationTimesChartProps,
) {
    let m = ChartMargins::default();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    draw_axes(context, width, height, &m);

    let points: Vec<(f64, f64)> = props
        .iteration_count
        .iter()
        .zip(props.measured_values.iter())
        .enumerate()
        .filter_map(|(idx, (iters, measured_ns))| {
            if *iters > 0.0 && iters.is_finite() && measured_ns.is_finite() {
                Some((idx as f64 + 1.0, (measured_ns / iters) / 1_000_000.0))
            } else {
                None
            }
        })
        .collect();

    if points.is_empty() {
        draw_axis_titles(
            context,
            width,
            height,
            &m,
            "Sample",
            "Average Iteration Time (ms)",
        );
        return;
    }

    let x_min = points.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let x_max = points
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = 0.0;
    let y_max = points
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(f64::EPSILON)
        * 1.1;

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
            min: y_min,
            max: y_max,
            num_lines: 5,
            fmt: format_tick_value,
        },
    );

    let area = PlotArea {
        width,
        height,
        x_min,
        x_max,
        y_min,
        y_max,
    };

    draw_scatter_points(context, &area, &points, &m);
    draw_axis_titles(
        context,
        width,
        height,
        &m,
        "Sample",
        "Average Iteration Time (ms)",
    );
}

fn draw_scatter_points(
    context: &CanvasRenderingContext2d,
    area: &PlotArea,
    points: &[(f64, f64)],
    m: &ChartMargins,
) {
    context.set_fill_style_str(PRIMARY_COLOR);
    for (x, y) in points {
        let px = map_x(*x, area.x_min, area.x_max, area.width, m);
        let py = map_y(*y, area.y_min, area.y_max, area.height, m);
        context.begin_path();
        let _ = context.arc(px, py, 2.5, 0.0, std::f64::consts::PI * 2.0);
        context.fill();
    }
}
