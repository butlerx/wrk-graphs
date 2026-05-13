#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
use crate::parser::criterion::ConfidenceInterval;
use gloo::events::EventListener;
use web_sys::{wasm_bindgen::JsCast, window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use crate::components::charts::chart_utils::{
    draw_axes, draw_axis_titles, draw_x_grid_and_labels, draw_y_grid_and_labels, format_tick_value,
    map_x, map_y, ChartMargins, GridConfig,
};

const BASELINE_COLOR: &str = "rgb(228, 26, 28)";
const BASELINE_BAND_COLOR: &str = "rgba(228, 26, 28, 0.1)";
const CURRENT_COLOR: &str = "rgb(31, 120, 180)";
const CURRENT_BAND_COLOR: &str = "rgba(31, 120, 180, 0.1)";

struct PlotArea {
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct CriterionRegressionComparisonChartProps {
    pub iteration_count: Vec<f64>,
    pub measured_values: Vec<f64>,
    pub slope: Option<ConfidenceInterval>,
    pub baseline_iteration_count: Vec<f64>,
    pub baseline_measured_values: Vec<f64>,
    pub baseline_slope: Option<ConfidenceInterval>,
}

#[function_component]
pub fn CriterionRegressionComparisonChart(props: &CriterionRegressionComparisonChartProps) -> Html {
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

                    draw_regression_comparison_chart(&context, width, height, &props_clone_resize);
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

fn draw_regression_comparison_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &CriterionRegressionComparisonChartProps,
) {
    let m = ChartMargins::default();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    draw_axes(context, width, height, &m);

    let current_points = compute_points(
        &props.iteration_count,
        &props.measured_values,
        props.slope.is_some(),
    );
    let baseline_points = compute_points(
        &props.baseline_iteration_count,
        &props.baseline_measured_values,
        props.baseline_slope.is_some(),
    );

    if current_points.len() < 2 && baseline_points.len() < 2 {
        draw_axis_titles(context, width, height, &m, "Iterations", "Total Time (ms)");
        return;
    }

    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for (x, y) in current_points.iter().chain(baseline_points.iter()) {
        x_min = x_min.min(*x);
        x_max = x_max.max(*x);
        y_max = y_max.max(*y);
    }

    let y_min = 0.0;
    y_max = y_max.max(f64::EPSILON) * 1.1;

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

    draw_scatter_points(context, &area, &baseline_points, BASELINE_COLOR, false, &m);
    if let Some(slope) = &props.baseline_slope {
        draw_regression_confidence_band(context, &area, slope, BASELINE_BAND_COLOR, &m);
        draw_regression_line(context, &area, slope.estimate, BASELINE_COLOR, &m);
    }

    draw_scatter_points(context, &area, &current_points, CURRENT_COLOR, true, &m);
    if let Some(slope) = &props.slope {
        draw_regression_confidence_band(context, &area, slope, CURRENT_BAND_COLOR, &m);
        draw_regression_line(context, &area, slope.estimate, CURRENT_COLOR, &m);
    }

    draw_legend(context, width, &m);
    draw_axis_titles(context, width, height, &m, "Iterations", "Total Time (ms)");
}

fn compute_points(
    iteration_count: &[f64],
    measured_values: &[f64],
    has_slope: bool,
) -> Vec<(f64, f64)> {
    if has_slope {
        iteration_count
            .iter()
            .zip(measured_values.iter())
            .filter_map(|(iters, measured_ns)| {
                if iters.is_finite() && measured_ns.is_finite() {
                    Some((*iters, measured_ns / 1_000_000.0))
                } else {
                    None
                }
            })
            .collect()
    } else {
        iteration_count
            .iter()
            .zip(measured_values.iter())
            .enumerate()
            .filter_map(|(idx, (iters, measured_ns))| {
                if *iters > 0.0 && iters.is_finite() && measured_ns.is_finite() {
                    Some((
                        f64::from(idx as u32) + 1.0,
                        (measured_ns / iters) / 1_000_000.0,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

fn draw_scatter_points(
    context: &CanvasRenderingContext2d,
    area: &PlotArea,
    points: &[(f64, f64)],
    color: &str,
    filled: bool,
    m: &ChartMargins,
) {
    if filled {
        context.set_fill_style_str(color);
    } else {
        context.set_stroke_style_str(color);
        context.set_line_width(1.0);
    }

    for (x, y) in points {
        let px = map_x(*x, area.x_min, area.x_max, area.width, m);
        let py = map_y(*y, area.y_min, area.y_max, area.height, m);
        context.begin_path();
        context
            .arc(px, py, 2.5, 0.0, std::f64::consts::PI * 2.0)
            .unwrap();
        if filled {
            context.fill();
        } else {
            context.stroke();
        }
    }
}

fn draw_regression_line(
    context: &CanvasRenderingContext2d,
    area: &PlotArea,
    slope_ns_per_iter: f64,
    color: &str,
    m: &ChartMargins,
) {
    let slope_ms_per_iter = slope_ns_per_iter / 1_000_000.0;

    let y1 = slope_ms_per_iter * area.x_min;
    let y2 = slope_ms_per_iter * area.x_max;

    context.set_stroke_style_str(color);
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
    color: &str,
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

    context.set_fill_style_str(color);
    context.begin_path();
    context.move_to(lx1, ly1);
    context.line_to(lx2, ly2);
    context.line_to(lx2, uy2);
    context.line_to(lx1, uy1);
    context.close_path();
    context.fill();
}

fn draw_legend(context: &CanvasRenderingContext2d, width: f64, m: &ChartMargins) {
    let legend_x = width - m.right - 80.0;
    let legend_y = m.top;

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
    context.fill_rect(legend_x, legend_y + 15.0, 10.0, 10.0);
    context.set_fill_style_str("black");
    context
        .fill_text("Current", legend_x + 15.0, legend_y + 20.0)
        .unwrap();
}
