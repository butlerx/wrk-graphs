use gloo::events::EventListener;
use web_sys::{window, CanvasRenderingContext2d};
use yew::prelude::*;

use super::chart_utils::setup_canvas;

/// Custom hook that sets up a canvas with high-DPI rendering and automatic resize handling.
///
/// Accepts a `draw` function that will be called with the canvas 2D context, width, and height
/// whenever the canvas needs to be redrawn (on mount and window resize).
///
/// Returns a `NodeRef` to attach to the `<canvas>` element.
#[hook]
pub fn use_canvas<F>(draw: F) -> NodeRef
where
    F: Fn(&CanvasRenderingContext2d, f64, f64) + 'static + Clone,
{
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        use_effect_with((), move |()| {
            let resize_callback = {
                let canvas_ref = canvas_ref.clone();
                move || {
                    if let Some((ctx, w, h)) = setup_canvas(&canvas_ref) {
                        draw(&ctx, w, h);
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

    canvas_ref
}
