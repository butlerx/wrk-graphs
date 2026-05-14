use web_sys::{wasm_bindgen::JsCast, window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::NodeRef;

/// Set up a canvas element for high-DPI rendering.
///
/// Casts the `NodeRef` to a `HtmlCanvasElement`, obtains the 2D context, resizes
/// the backing store to match the parent element width (with a 0.6 aspect ratio),
/// resets the transform, and scales for the device pixel ratio.
///
/// Returns `None` if any step fails (e.g. canvas not yet mounted), allowing the
/// caller to silently skip rendering instead of panicking.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub fn setup_canvas(canvas_ref: &NodeRef) -> Option<(CanvasRenderingContext2d, f64, f64)> {
    let canvas = canvas_ref.cast::<HtmlCanvasElement>()?;
    let context = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;

    let dpr = window()?.device_pixel_ratio();
    let parent = canvas.parent_element()?;
    let width = f64::from(parent.client_width());
    let height = width * 0.6;

    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);

    // Reset transform before scaling to prevent DPR compounding on resize.
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).ok()?;
    context.scale(dpr, dpr).ok()?;

    Some((context, width, height))
}

/// Asymmetric margins for canvas charts — enough room for tick labels + axis titles.
pub struct ChartMargins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for ChartMargins {
    fn default() -> Self {
        Self {
            top: 20.0,
            right: 25.0,
            bottom: 64.0,
            left: 72.0,
        }
    }
}

impl ChartMargins {
    /// Width of the plot area (canvas width minus left and right margins).
    pub fn plot_width(&self, canvas_width: f64) -> f64 {
        canvas_width - self.left - self.right
    }

    /// Height of the plot area (canvas height minus top and bottom margins).
    pub fn plot_height(&self, canvas_height: f64) -> f64 {
        canvas_height - self.top - self.bottom
    }

    /// Compact margins for smaller mini-charts (stat distributions).
    pub fn compact() -> Self {
        Self {
            top: 24.0,
            right: 20.0,
            bottom: 54.0,
            left: 56.0,
        }
    }
}

/// Map a data-space X value to canvas pixel X.
pub fn map_x(value: f64, x_min: f64, x_max: f64, canvas_width: f64, m: &ChartMargins) -> f64 {
    if (x_max - x_min).abs() < f64::EPSILON {
        m.left + m.plot_width(canvas_width) / 2.0
    } else {
        m.left + ((value - x_min) / (x_max - x_min)) * m.plot_width(canvas_width)
    }
}

/// Map a data-space Y value to canvas pixel Y (Y increases downward).
pub fn map_y(value: f64, y_min: f64, y_max: f64, canvas_height: f64, m: &ChartMargins) -> f64 {
    if (y_max - y_min).abs() < f64::EPSILON {
        canvas_height - m.bottom
    } else {
        canvas_height
            - m.bottom
            - ((value - y_min) / (y_max - y_min)) * m.plot_height(canvas_height)
    }
}

/// Draw X and Y axis lines.
pub fn draw_axes(ctx: &CanvasRenderingContext2d, w: f64, h: f64, m: &ChartMargins) {
    ctx.set_stroke_style_str("#cccccc");
    ctx.set_line_width(1.0);
    // X axis
    ctx.begin_path();
    ctx.move_to(m.left, h - m.bottom);
    ctx.line_to(w - m.right, h - m.bottom);
    ctx.stroke();
    // Y axis
    ctx.begin_path();
    ctx.move_to(m.left, m.top);
    ctx.line_to(m.left, h - m.bottom);
    ctx.stroke();
}

/// Draw axis titles. X title centered below tick labels, Y title rotated left of tick labels.
pub fn draw_axis_titles(
    ctx: &CanvasRenderingContext2d,
    w: f64,
    h: f64,
    m: &ChartMargins,
    x_label: &str,
    y_label: &str,
) {
    ctx.set_fill_style_str("#111");
    ctx.set_font("bold 12px monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("bottom");
    let _ = ctx.fill_text(x_label, (m.left + w - m.right) / 2.0, h - 4.0);

    ctx.save();
    ctx.set_text_baseline("middle");
    let _ = ctx.translate(12.0, (m.top + h - m.bottom) / 2.0);
    let _ = ctx.rotate(-std::f64::consts::PI / 2.0);
    let _ = ctx.fill_text(y_label, 0.0, 0.0);
    ctx.restore();
}

/// Parameters for drawing grid lines and tick labels along an axis.
pub struct GridConfig<F: Fn(f64) -> String> {
    pub min: f64,
    pub max: f64,
    pub num_lines: i32,
    pub fmt: F,
}

/// Draw vertical grid lines + X-axis tick labels.
///
/// Labels are placed 6px below the axis line with baseline "top",
/// well above the axis title at the canvas bottom.
pub fn draw_x_grid_and_labels(
    ctx: &CanvasRenderingContext2d,
    canvas_width: f64,
    canvas_height: f64,
    margins: &ChartMargins,
    cfg: &GridConfig<impl Fn(f64) -> String>,
) {
    ctx.set_stroke_style_str("#e0e0e0");
    ctx.set_line_width(1.0);
    ctx.set_fill_style_str("#444");
    ctx.set_font("10px monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");

    let pw = margins.plot_width(canvas_width);
    for i in 0..=cfg.num_lines {
        let t = f64::from(i) / f64::from(cfg.num_lines);
        let x_value = cfg.min + t * (cfg.max - cfg.min);
        let px = margins.left + t * pw;

        ctx.begin_path();
        ctx.move_to(px, margins.top);
        ctx.line_to(px, canvas_height - margins.bottom);
        ctx.stroke();

        let _ = ctx.fill_text(
            &(cfg.fmt)(x_value),
            px,
            canvas_height - margins.bottom + 10.0,
        );
    }
}

/// Draw horizontal grid lines + Y-axis tick labels.
///
/// Labels are placed 8px left of the axis line with alignment "right".
pub fn draw_y_grid_and_labels(
    ctx: &CanvasRenderingContext2d,
    canvas_width: f64,
    canvas_height: f64,
    margins: &ChartMargins,
    cfg: &GridConfig<impl Fn(f64) -> String>,
) {
    ctx.set_stroke_style_str("#e0e0e0");
    ctx.set_line_width(1.0);
    ctx.set_fill_style_str("#444");
    ctx.set_font("10px monospace");
    ctx.set_text_align("right");
    ctx.set_text_baseline("middle");

    let ph = margins.plot_height(canvas_height);
    let step_value = (cfg.max - cfg.min) / f64::from(cfg.num_lines);
    let step_px = ph / f64::from(cfg.num_lines);

    for i in 0..=cfg.num_lines {
        let py = canvas_height - margins.bottom - f64::from(i) * step_px;

        ctx.begin_path();
        ctx.move_to(margins.left, py);
        ctx.line_to(canvas_width - margins.right, py);
        ctx.stroke();

        let value = cfg.min + f64::from(i) * step_value;
        let _ = ctx.fill_text(&(cfg.fmt)(value), margins.left - 10.0, py);
    }
}

/// Smart tick label formatter — keeps labels concise to avoid overflow.
pub fn format_tick_value(value: f64) -> String {
    let abs = value.abs();
    if abs < f64::EPSILON {
        "0".to_string()
    } else if abs >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if abs >= 10_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else if abs >= 100.0 {
        format!("{value:.1}")
    } else if abs >= 1.0 {
        format!("{value:.2}")
    } else if abs >= 0.01 {
        format!("{value:.3}")
    } else {
        format!("{value:.2e}")
    }
}
