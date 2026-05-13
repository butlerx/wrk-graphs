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

                    draw_iteration_times_chart(&context, width, height, &props_clone_resize);
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
        context
            .arc(px, py, 2.5, 0.0, std::f64::consts::PI * 2.0)
            .unwrap();
        context.fill();
    }
}
