#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
use gloo::events::EventListener;
use web_sys::{window, CanvasRenderingContext2d};
use yew::prelude::*;

use super::chart_utils::{self, map_y, setup_canvas, ChartMargins};

#[derive(Clone, Debug, PartialEq, PartialOrd, Properties, Default)]
pub struct LineCurveChartConfig {
    #[prop_or(true)]
    pub show_inflection_points: bool,
    #[prop_or_default]
    pub stroke_width: i32,
    #[prop_or(false)]
    pub show_area_chart: bool,
    #[prop_or(String::new())]
    pub x_axis_title: String,
    #[prop_or(String::new())]
    pub y_axis_title: String,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Properties)]
pub struct Series {
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Properties)]
pub struct LineCurveChartProps {
    pub data: Vec<(Series, Vec<(f64, f64)>)>,
    pub x: Vec<String>,
    #[prop_or_default]
    pub config: LineCurveChartConfig,
}

#[function_component]
pub fn LineCurveChart(props: &LineCurveChartProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let props_clone = props.clone();
        use_effect_with((), move |()| {
            let resize_callback = {
                let canvas_ref = canvas_ref.clone();
                move || {
                    if let Some((ctx, w, h)) = setup_canvas(&canvas_ref) {
                        draw_multiline_chart(&ctx, w, h, &props_clone);
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
            <canvas ref={canvas_ref} role="img" aria-label="Line chart visualization" style="width: 100%; height: 100%; box-sizing: border-box" />
            <div
                style="
                    position: absolute;
                    top: 10px;
                    right: 10px;
                    background-color: rgba(255, 255, 255, 0.9);
                    padding: 5px;
                    border-radius: 4px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1)
                "
            >
                <div style="display: flex; flex-direction: row; gap: 5px; flex-wrap: wrap">
                    { for props.data.iter().map(|(series, _)| {
                            html! {
                                <div style="display: flex; flex-direction: row; align-items: center; gap: 2px;">
                                    <span style="font-size: 10px;">{ &series.name }</span>
                                    <div style={format!("background-color: {}; width: 10px; height: 10px; display: inline-block;", series.color)}></div>
                                </div>
                            }
                        }) }
                </div>
            </div>
        </div>
    }
}

fn draw_multiline_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &LineCurveChartProps,
) {
    let m = ChartMargins::default();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    let max_value = get_max_value(props);
    let point_spacing = m.plot_width(width) / 100.0;

    chart_utils::draw_axes(context, width, height, &m);
    draw_vertical_grid_lines(context, height, &m, point_spacing);
    chart_utils::draw_y_grid_and_labels(
        context,
        width,
        height,
        &m,
        &chart_utils::GridConfig {
            min: 0.0,
            max: max_value,
            num_lines: 5,
            fmt: chart_utils::format_tick_value,
        },
    );
    draw_datasets(context, height, &m, point_spacing, props, max_value);
    draw_x_axis_labels(context, height, &m, point_spacing, props);
    if !props.config.x_axis_title.is_empty() || !props.config.y_axis_title.is_empty() {
        chart_utils::draw_axis_titles(
            context,
            width,
            height,
            &m,
            &props.config.x_axis_title,
            &props.config.y_axis_title,
        );
    }
}

fn get_max_value(props: &LineCurveChartProps) -> f64 {
    props
        .data
        .iter()
        .flat_map(|(_, data)| data.iter().map(|(_, y)| *y))
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0)
        * 1.2
}

/// Custom vertical grid lines — spaced by `point_spacing` (every 10th point),
/// since X values are index-based, not continuous data-space values.
fn draw_vertical_grid_lines(
    context: &CanvasRenderingContext2d,
    height: f64,
    m: &ChartMargins,
    point_spacing: f64,
) {
    context.set_stroke_style_str("#e0e0e0");
    context.set_line_width(1.0);
    for i in 0..=10 {
        let x = m.left + (f64::from(i) * 10.0) * point_spacing;
        context.begin_path();
        context.move_to(x, height - m.bottom);
        context.line_to(x, m.top);
        context.stroke();
    }
}

fn draw_datasets(
    context: &CanvasRenderingContext2d,
    height: f64,
    m: &ChartMargins,
    point_spacing: f64,
    props: &LineCurveChartProps,
    max_value: f64,
) {
    let datasets = &props.data;
    for (series, data) in datasets {
        let Some(first_point) = data.first() else {
            continue;
        };
        context.set_stroke_style_str(series.color.as_str());
        context.set_line_width(f64::from(props.config.stroke_width));
        context.begin_path();
        let first_x = m.left + first_point.0 * point_spacing;
        let first_y = map_y(first_point.1, 0.0, max_value, height, m);
        context.move_to(first_x, first_y);
        for i in 1..data.len() {
            let point = &data[i];
            let prev_point = &data[i - 1];
            let x = m.left + point.0 * point_spacing;
            let y = map_y(point.1, 0.0, max_value, height, m);
            let prev_x = m.left + prev_point.0 * point_spacing;
            let prev_y = map_y(prev_point.1, 0.0, max_value, height, m);
            let dx = x - prev_x;
            let ctrl1_x = prev_x + dx * 0.5;
            let ctrl1_y = prev_y;
            let ctrl2_x = x - dx * 0.5;
            let ctrl2_y = y;
            context.bezier_curve_to(ctrl1_x, ctrl1_y, ctrl2_x, ctrl2_y, x, y);
        }
        context.stroke();
        if props.config.show_area_chart {
            if let Some(last_point) = data.last() {
                context.line_to(m.left + last_point.0 * point_spacing, height - m.bottom);
                context.line_to(m.left, height - m.bottom);
                context.close_path();
                let fill_color = format!("{}33", &series.color);
                context.set_fill_style_str(&fill_color);
                context.fill();
            }
        }
        if props.config.show_inflection_points {
            context.set_fill_style_str(series.color.as_str());
            for point in data {
                let x = m.left + point.0 * point_spacing;
                let y = map_y(point.1, 0.0, max_value, height, m);
                context.begin_path();
                let _ = context.arc(x, y, 2.0, 1.0, std::f64::consts::PI * 2.0);
                context.fill();
                context.set_fill_style_str(series.color.as_str());
                context.set_font("0.5em monospace");
                context.set_text_align("center");
                let y_offset = if point.0 % 20.0 < 10.0 { -4.0 } else { 8.0 };
                context.set_text_baseline(if point.0 % 20.0 < 10.0 {
                    "bottom"
                } else {
                    "top"
                });
                let _ = context.fill_text(&format!("{:.2}", point.1), x, y + y_offset);
            }
        }
        context.set_font("10px monospace");
    }
}

/// Custom X-axis labels — positioned by `point_spacing` intervals,
/// reading from `props.x` string labels (not numeric data values).
fn draw_x_axis_labels(
    context: &CanvasRenderingContext2d,
    height: f64,
    m: &ChartMargins,
    point_spacing: f64,
    props: &LineCurveChartProps,
) {
    context.set_fill_style_str("#444");
    context.set_font("10px monospace");
    context.set_text_align("center");
    context.set_text_baseline("top");
    let x_labels = props.x.clone().into_iter();
    for (i, x_label) in x_labels.enumerate() {
        let x = m.left + (f64::from(i as u32) * 10.0) * point_spacing;
        let y = height - m.bottom + 10.0;
        let _ = context.fill_text(x_label.as_str(), x, y);
    }
}
