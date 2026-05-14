#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
use crate::parser::criterion::ConfidenceInterval;
use gloo::events::EventListener;
use web_sys::{window, CanvasRenderingContext2d};
use yew::prelude::*;

use crate::components::charts::chart_utils::{
    draw_axes, draw_axis_titles, draw_x_grid_and_labels, draw_y_grid_and_labels, format_tick_value,
    map_x, map_y, setup_canvas, ChartMargins, GridConfig,
};
use crate::components::charts::data_utils::compute_regression_points;

const PRIMARY_COLOR: &str = "rgb(31, 120, 180)";
const BAND_COLOR: &str = "rgba(31, 120, 180, 0.2)";

struct PlotArea {
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct CriterionRegressionChartProps {
    pub iteration_count: Vec<f64>,
    pub measured_values: Vec<f64>,
    pub slope: Option<ConfidenceInterval>,
}

#[function_component]
pub fn CriterionRegressionChart(props: &CriterionRegressionChartProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let props_clone = props.clone();
        use_effect_with((), move |()| {
            let resize_callback = {
                let canvas_ref = canvas_ref.clone();
                let props = props_clone.clone();
                move || {
                    if let Some((context, width, height)) = setup_canvas(&canvas_ref) {
                        draw_regression_chart(&context, width, height, &props);
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
            <canvas ref={canvas_ref} role="img" aria-label="Regression analysis chart" style="width: 100%; height: 100%; box-sizing: border-box" />
        </div>
    }
}

fn draw_regression_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &CriterionRegressionChartProps,
) {
    let m = ChartMargins::default();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    draw_axes(context, width, height, &m);

    let points = compute_regression_points(
        &props.iteration_count,
        &props.measured_values,
        props.slope.is_some(),
    );
    if points.len() < 2 {
        draw_axis_titles(context, width, height, &m, "Iterations", "Total Time (ms)");
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

    if let Some(slope) = &props.slope {
        draw_regression_confidence_band(context, &area, slope, &m);
        draw_regression_line(context, &area, slope.estimate, &m);
    }

    draw_axis_titles(context, width, height, &m, "Iterations", "Total Time (ms)");
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

fn draw_regression_line(
    context: &CanvasRenderingContext2d,
    area: &PlotArea,
    slope_ns_per_iter: f64,
    m: &ChartMargins,
) {
    let slope_ms_per_iter = slope_ns_per_iter / 1_000_000.0;

    let y1 = slope_ms_per_iter * area.x_min;
    let y2 = slope_ms_per_iter * area.x_max;

    context.set_stroke_style_str(PRIMARY_COLOR);
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(
        map_x(area.x_min, area.x_min, area.x_max, area.width, m),
        map_y(y1, area.y_min, area.y_max, area.height, m),
    );
    context.line_to(
        map_x(area.x_max, area.x_min, area.x_max, area.width, m),
        map_y(y2, area.y_min, area.y_max, area.height, m),
    );
    context.stroke();
}

fn draw_regression_confidence_band(
    context: &CanvasRenderingContext2d,
    area: &PlotArea,
    slope: &ConfidenceInterval,
    m: &ChartMargins,
) {
    let lower_ms = slope.lower_bound / 1_000_000.0;
    let upper_ms = slope.upper_bound / 1_000_000.0;

    let lx1 = map_x(area.x_min, area.x_min, area.x_max, area.width, m);
    let lx2 = map_x(area.x_max, area.x_min, area.x_max, area.width, m);
    let ly1 = map_y(
        lower_ms * area.x_min,
        area.y_min,
        area.y_max,
        area.height,
        m,
    );
    let ly2 = map_y(
        lower_ms * area.x_max,
        area.y_min,
        area.y_max,
        area.height,
        m,
    );

    let uy1 = map_y(
        upper_ms * area.x_min,
        area.y_min,
        area.y_max,
        area.height,
        m,
    );
    let uy2 = map_y(
        upper_ms * area.x_max,
        area.y_min,
        area.y_max,
        area.height,
        m,
    );

    context.set_fill_style_str(BAND_COLOR);
    context.begin_path();
    context.move_to(lx1, ly1);
    context.line_to(lx2, ly2);
    context.line_to(lx2, uy2);
    context.line_to(lx1, uy1);
    context.close_path();
    context.fill();
}
