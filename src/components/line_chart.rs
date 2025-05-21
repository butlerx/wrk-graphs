use gloo::events::EventListener;
use web_sys::{wasm_bindgen::JsCast, window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

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
                    // let height = parent.client_height() as f64;
                    let height = width * 0.6;

                    // Set the canvas dimensions to match its parent's dimensions
                    // Set the canvas dimensions to match its parent's dimensions
                    canvas.set_width((width * device_pixel_ratio) as u32);
                    canvas.set_height((height * device_pixel_ratio) as u32);

                    // Scale the context to account for the device pixel ratio
                    context
                        .scale(device_pixel_ratio, device_pixel_ratio)
                        .unwrap();

                    draw_multiline_chart(&context, width, height, &props_clone_resize);
                }
            };

            resize_callback(); // Initial call to set canvas size

            let listener = EventListener::new(&window().unwrap(), "resize", move |_event| {
                resize_callback();
            });

            move || drop(listener) // Clean up the event listener on component unmount
        });
    }

    html! {
        <div style="position: relative">
            <canvas ref={canvas_ref} style="width: 100%; height: 100%; box-sizing: border-box" />
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
    let axis_padding = 50.0;
    let max_value = get_max_value(props);
    let point_spacing = (width - axis_padding * 2.0) / 100.0;

    draw_axes(context, width, height, axis_padding);
    draw_vertical_grid_lines(context, height, axis_padding, point_spacing);
    draw_horizontal_grid_lines_and_labels(context, width, height, axis_padding, max_value);
    draw_datasets(
        context,
        height,
        axis_padding,
        point_spacing,
        props,
        max_value,
    );
    draw_x_axis_labels(context, height, axis_padding, point_spacing, props);
    draw_axis_titles(context, width, height, axis_padding, &props.config);
}

fn get_max_value(props: &LineCurveChartProps) -> f64 {
    props
        .data
        .iter()
        .flat_map(|(_, data)| data.iter().map(|(_, y)| *y))
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
        * 1.2
}

fn draw_axes(context: &CanvasRenderingContext2d, width: f64, height: f64, axis_padding: f64) {
    context.set_stroke_style_str("#cccccc");
    context.set_line_width(1.0);
    // x-axis
    context.begin_path();
    context.move_to(axis_padding, height - axis_padding);
    context.line_to(width - axis_padding, height - axis_padding);
    context.stroke();
    // y-axis
    context.begin_path();
    context.move_to(axis_padding, 0.0);
    context.line_to(axis_padding, height - axis_padding);
    context.stroke();
}

fn draw_vertical_grid_lines(
    context: &CanvasRenderingContext2d,
    height: f64,
    axis_padding: f64,
    point_spacing: f64,
) {
    // Convert max_y to canvas y position
    let max_y_pos = height - axis_padding * (height - axis_padding * 2.0);
    context.set_stroke_style_str("#cccccc");
    context.set_line_width(1.0);
    for i in 0..=10 {
        let x = axis_padding + (f64::from(i) * 10.0) * point_spacing;
        context.begin_path();
        context.move_to(x, height - axis_padding);
        context.line_to(x, max_y_pos);
        context.stroke();
    }
}

fn draw_horizontal_grid_lines_and_labels(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    axis_padding: f64,
    max_value: f64,
) {
    context.set_font("8px monospace");
    context.set_stroke_style_str("#cccccc");
    context.set_line_width(1.0);
    context.set_fill_style_str("black");
    context.set_text_align("right");
    context.set_text_baseline("middle");
    let num_grid_lines = 5;
    let step_value = max_value / f64::from(num_grid_lines);
    let step_height = (height - axis_padding * 2.0) / f64::from(num_grid_lines);
    for i in 0..=num_grid_lines {
        let y = height - axis_padding - f64::from(i) * step_height;
        context.begin_path();
        context.move_to(axis_padding, y);
        context.line_to(width - axis_padding, y);
        context.stroke();
        let label = f64::from(i) * step_value;
        context
            .fill_text(&format!("{label:.2}"), axis_padding - 10.0, y)
            .unwrap();
    }
}

fn draw_datasets(
    context: &CanvasRenderingContext2d,
    height: f64,
    axis_padding: f64,
    point_spacing: f64,
    props: &LineCurveChartProps,
    max_value: f64,
) {
    let datasets = &props.data;
    for (series, data) in datasets {
        context.set_stroke_style_str(series.color.as_str());
        context.set_line_width(f64::from(props.config.stroke_width));
        context.begin_path();
        let first_point = data.first().unwrap();
        let first_x = axis_padding + first_point.0 * point_spacing;
        let first_y =
            height - axis_padding - (first_point.1 / max_value) * (height - axis_padding * 2.0);
        context.move_to(first_x, first_y);
        for i in 1..data.len() {
            let point = &data[i];
            let prev_point = &data[i - 1];
            let x = axis_padding + point.0 * point_spacing;
            let y = height - axis_padding - (point.1 / max_value) * (height - axis_padding * 2.0);
            let prev_x = axis_padding + prev_point.0 * point_spacing;
            let prev_y =
                height - axis_padding - (prev_point.1 / max_value) * (height - axis_padding * 2.0);
            let dx = x - prev_x;
            let ctrl1_x = prev_x + dx * 0.5;
            let ctrl1_y = prev_y;
            let ctrl2_x = x - dx * 0.5;
            let ctrl2_y = y;
            context.bezier_curve_to(ctrl1_x, ctrl1_y, ctrl2_x, ctrl2_y, x, y);
        }
        context.stroke();
        if props.config.show_area_chart {
            let last_point = data.last().unwrap();
            context.line_to(
                axis_padding + last_point.0 * point_spacing,
                height - axis_padding,
            );
            context.line_to(axis_padding, height - axis_padding);
            context.close_path();
            let fill_color = format!("{}33", &series.color); // Lighter shade (transparent)
            context.set_fill_style_str(&fill_color);
            context.fill();
        }
        if props.config.show_inflection_points {
            context.set_fill_style_str(series.color.as_str());
            for point in data {
                let x = axis_padding + point.0 * point_spacing;
                let y =
                    height - axis_padding - (point.1 / max_value) * (height - axis_padding * 2.0);
                context.begin_path();
                context
                    .arc(x, y, 2.0, 1.0, std::f64::consts::PI * 2.0)
                    .unwrap();
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
                context
                    .fill_text(&format!("{:.2}", point.1), x, y + y_offset)
                    .unwrap();
            }
        }
        context.set_font("8px monospace");
    }
}

fn draw_x_axis_labels(
    context: &CanvasRenderingContext2d,
    height: f64,
    axis_padding: f64,
    point_spacing: f64,
    props: &LineCurveChartProps,
) {
    context.set_fill_style_str("black");
    context.set_text_align("center");
    context.set_text_baseline("middle");
    let x_labels = props.x.clone().into_iter();
    for (i, x_label) in x_labels.enumerate() {
        let x = axis_padding + (i as f64 * 10.0) * point_spacing;
        let y = height - axis_padding / 2.0;
        context.fill_text(x_label.as_str(), x, y).unwrap();
    }
}

fn draw_axis_titles(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    axis_padding: f64,
    config: &LineCurveChartConfig,
) {
    if !config.x_axis_title.is_empty() {
        context.set_text_align("center");
        context.set_font("bold 12px monospace");
        context
            .fill_text(
                &config.x_axis_title,
                width / 2.0,
                height - (axis_padding / 4.0),
            )
            .unwrap();
    }
    if !config.y_axis_title.is_empty() {
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context.set_font("bold 12px monospace");
        context.save();
        context.rotate(-std::f64::consts::PI / 2.0).unwrap();
        context
            .fill_text(&config.y_axis_title, -(height / 2.0), axis_padding / 4.0)
            .unwrap();
        context.restore();
    }
}
