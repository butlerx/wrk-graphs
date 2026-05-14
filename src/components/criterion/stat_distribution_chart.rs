#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]

use crate::parser::criterion::ConfidenceInterval;
use web_sys::CanvasRenderingContext2d;
use yew::prelude::*;

use crate::components::charts::chart_utils::{
    draw_axes, draw_x_grid_and_labels, map_x, map_y, ChartMargins, GridConfig,
};
use crate::components::charts::use_canvas;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct CriterionStatDistributionChartProps {
    pub ci: ConfidenceInterval,
    pub label: AttrValue,
}

struct MiniPlotArea {
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
    y_max: f64,
}

#[function_component]
pub fn CriterionStatDistributionChart(props: &CriterionStatDistributionChartProps) -> Html {
    let props_clone = props.clone();
    let canvas_ref = use_canvas(move |ctx, w, h| {
        draw_chart(ctx, w, h, &props_clone);
    });

    html! {
        <div style="position: relative; width: 100%">
            <canvas
                ref={canvas_ref}
                role="img"
                aria-label="Statistical distribution chart"
                style="width: 100%; display: block"
            />
        </div>
    }
}

fn draw_chart(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    props: &CriterionStatDistributionChartProps,
) {
    let m = ChartMargins::compact();
    context.set_fill_style_str("white");
    context.fill_rect(0.0, 0.0, width, height);

    let ci = &props.ci;

    let mean = ci.estimate;
    let mut std = (ci.upper_bound - ci.lower_bound) / (2.0 * 1.96);
    if std <= 0.0 || !std.is_finite() {
        std = (mean.abs() * 0.01).max(1e-9);
    }

    let x_min = mean - 4.0 * std;
    let x_max = mean + 4.0 * std;

    let points = 150;
    let mut kde_points = Vec::with_capacity(points);
    let step = (x_max - x_min) / (points as f64 - 1.0);
    let mut y_max = 0.0;

    let norm = 1.0 / (std * (2.0 * std::f64::consts::PI).sqrt());

    for i in 0..points {
        let x = x_min + i as f64 * step;
        let exponent = -0.5 * ((x - mean) / std).powi(2);
        let y = norm * exponent.exp();
        if y > y_max {
            y_max = y;
        }
        kde_points.push((x, y));
    }
    y_max *= 1.1;

    let area = MiniPlotArea {
        width,
        height,
        x_min,
        x_max,
        y_max,
    };

    draw_axes(context, width, height, &m);
    let unit = ci.unit.clone();
    draw_x_grid_and_labels(
        context,
        width,
        height,
        &m,
        &GridConfig {
            min: x_min,
            max: x_max,
            num_lines: 5,
            fmt: |v| format!("{v:.3}{}", unit.as_str()),
        },
    );
    draw_ci_band(context, &area, ci, &m);
    draw_distribution_area(context, &area, &kde_points, &m);
    draw_distribution_line(context, &area, &kde_points, &m);
    draw_estimate_line(context, &area, ci.estimate, &m);
    draw_labels(context, width, height, &m, &props.label);
}

fn draw_ci_band(
    context: &CanvasRenderingContext2d,
    area: &MiniPlotArea,
    ci: &ConfidenceInterval,
    m: &ChartMargins,
) {
    let x_start = map_x(ci.lower_bound, area.x_min, area.x_max, area.width, m);
    let x_end = map_x(ci.upper_bound, area.x_min, area.x_max, area.width, m);

    context.set_fill_style_str("rgba(31, 120, 180, 0.1)");
    context.fill_rect(x_start, m.top, x_end - x_start, m.plot_height(area.height));
}

fn draw_distribution_area(
    context: &CanvasRenderingContext2d,
    area: &MiniPlotArea,
    points: &[(f64, f64)],
    m: &ChartMargins,
) {
    context.set_fill_style_str("rgba(31, 120, 180, 0.2)");
    context.begin_path();

    if let Some((first_x, _)) = points.first() {
        context.move_to(
            map_x(*first_x, area.x_min, area.x_max, area.width, m),
            area.height - m.bottom,
        );
    }

    for (x, y) in points {
        context.line_to(
            map_x(*x, area.x_min, area.x_max, area.width, m),
            map_y(*y, 0.0, area.y_max, area.height, m),
        );
    }

    if let Some((last_x, _)) = points.last() {
        context.line_to(
            map_x(*last_x, area.x_min, area.x_max, area.width, m),
            area.height - m.bottom,
        );
    }

    context.close_path();
    context.fill();
}

fn draw_distribution_line(
    context: &CanvasRenderingContext2d,
    area: &MiniPlotArea,
    points: &[(f64, f64)],
    m: &ChartMargins,
) {
    context.set_stroke_style_str("rgb(31, 120, 180)");
    context.set_line_width(2.0);
    context.begin_path();

    for (i, (x, y)) in points.iter().enumerate() {
        let px = map_x(*x, area.x_min, area.x_max, area.width, m);
        let py = map_y(*y, 0.0, area.y_max, area.height, m);
        if i == 0 {
            context.move_to(px, py);
        } else {
            context.line_to(px, py);
        }
    }
    context.stroke();
}

fn draw_estimate_line(
    context: &CanvasRenderingContext2d,
    area: &MiniPlotArea,
    estimate: f64,
    m: &ChartMargins,
) {
    let x = map_x(estimate, area.x_min, area.x_max, area.width, m);
    context.set_stroke_style_str("rgb(31, 120, 180)");
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(x, m.top);
    context.line_to(x, area.height - m.bottom);
    context.stroke();
}

fn draw_labels(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    m: &ChartMargins,
    label: &str,
) {
    context.set_fill_style_str("#111");
    context.set_font("bold 11px monospace");
    context.set_text_align("center");
    context.set_text_baseline("middle");
    let _ = context.fill_text(label, width / 2.0, m.top / 2.0);

    context.save();
    let _ = context.translate(12.0, (m.top + height - m.bottom) / 2.0);
    let _ = context.rotate(-std::f64::consts::PI / 2.0);
    context.set_font("10px monospace");
    let _ = context.fill_text("Density", 0.0, 0.0);
    context.restore();
}
