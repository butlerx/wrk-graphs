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

                    draw_pdf_chart(&context, width, height, &props_clone_resize);
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
