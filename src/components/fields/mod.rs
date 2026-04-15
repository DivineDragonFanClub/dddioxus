//! Reusable numeric / value fields for the Inspector and beyond.
//!
//! `DragFloat` is the single-float primitive (hover-drag, shift/ctrl
//! modifiers, canonical formatting, optional wrapping). `Vec3Editor`
//! lives in its own submodule and composes three `DragFloat`s plus
//! an OS-clipboard copy/paste menu.

use dioxus::prelude::*;

pub mod vec3;
pub use vec3::Vec3Editor;

/// Units per pixel of horizontal drag. Matches Unity's default feel
/// for float fields (~0.03 per pixel).
const DEFAULT_DRAG_SENSITIVITY: f32 = 0.03;
/// Shift: 10x — coarse drag for broad strokes.
const COARSE_MULTIPLIER: f32 = 10.0;
/// Ctrl/Cmd: 0.1x — fine drag for precision.
const FINE_MULTIPLIER: f32 = 0.1;

/// Format a float with 4 decimal places, always. No trailing-zero
/// trim — keeping the fixed width makes vertical alignment across
/// rows stable and matches Unity's default transform inspector look.
pub(crate) fn fmt_float(v: f32) -> String {
    format!("{:.4}", v)
}

/// If `wrap` is `Some(w)` with `w > 0`, normalize `v` into `[0, w)`.
/// Used for angle fields — rotation values stay within 0..360 regardless
/// of how far the user drags.
pub(crate) fn apply_wrap(v: f32, wrap: Option<f32>) -> f32 {
    match wrap {
        Some(w) if w > 0.0 => ((v % w) + w) % w,
        _ => v,
    }
}

/// A single-float numeric input with a drag handle.
///
/// The `label` is rendered as a small colored letter before the input;
/// hovering it shows the horizontal-resize cursor, and click-and-drag
/// changes the value live. Typing + Enter/blur also works.
///
/// Fires `on_change` on every drag frame and on every text commit.
/// The underlying text state is local to this component, so containers
/// that are fire-and-forget (like `TransformInspector`) stay in sync
/// with what the user is editing without needing to refetch.
#[derive(PartialEq, Clone, Props)]
pub struct DragFloatProps {
    pub value: f32,
    pub on_change: EventHandler<f32>,
    #[props(default = "")]
    pub label: &'static str,
    #[props(default = "text-gray-400")]
    pub color: &'static str,
    #[props(default = DEFAULT_DRAG_SENSITIVITY)]
    pub sensitivity: f32,
    /// If `Some(w)`, the value is wrapped into `[0, w)` before display
    /// and before `on_change` fires. Intended for angle fields.
    #[props(default)]
    pub wrap: Option<f32>,
}

#[component]
pub fn DragFloat(props: DragFloatProps) -> Element {
    let wrap = props.wrap;
    let mut text = use_signal(|| fmt_float(apply_wrap(props.value, wrap)));
    let mut last_value = use_signal(|| props.value);
    // (last_x, accumulated_value) — we accumulate frame-by-frame so holding
    // shift/ctrl mid-drag just changes pace from that point on, with no
    // discontinuity in the displayed value. The accumulator stores the
    // raw (unwrapped) value; we only wrap on display and on fire.
    let mut drag_state = use_signal(|| None::<(f64, f32)>);

    if (*last_value.read() - props.value).abs() > f32::EPSILON {
        last_value.set(props.value);
        text.set(fmt_float(apply_wrap(props.value, wrap)));
    }

    let on_change = props.on_change;
    let sensitivity = props.sensitivity;
    let fallback = props.value;

    let submit = move |_| {
        let parsed = text().parse::<f32>().unwrap_or(fallback);
        let wrapped = apply_wrap(parsed, wrap);
        text.set(fmt_float(wrapped));
        on_change.call(wrapped);
    };

    let dragging = drag_state().is_some();
    let input_class = "w-20 px-1 py-0.5 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center";

    rsx! {
        if !props.label.is_empty() {
            span {
                class: "{props.color} text-xs font-bold w-3 select-none",
                style: "cursor: ew-resize",
                onmousedown: move |e: Event<MouseData>| {
                    e.stop_propagation();
                    let start = text().parse::<f32>().unwrap_or(fallback);
                    drag_state.set(Some((e.client_coordinates().x, start)));
                },
                "{props.label}"
            }
        }
        input {
            class: "{input_class}",
            value: "{text}",
            oninput: move |e| text.set(e.value()),
            onchange: submit,
        }
        if dragging {
            div {
                class: "fixed inset-0 z-50",
                style: "cursor: ew-resize",
                onmousemove: move |e: Event<MouseData>| {
                    if let Some((last_x, current_val)) = drag_state() {
                        let new_x = e.client_coordinates().x;
                        let delta = (new_x - last_x) as f32;
                        let mods = e.modifiers();
                        let mult = if mods.contains(Modifiers::SHIFT) {
                            COARSE_MULTIPLIER
                        } else if mods.contains(Modifiers::CONTROL)
                            || mods.contains(Modifiers::META)
                        {
                            FINE_MULTIPLIER
                        } else {
                            1.0
                        };
                        let new_val = current_val + delta * sensitivity * mult;
                        drag_state.set(Some((new_x, new_val)));
                        let wrapped = apply_wrap(new_val, wrap);
                        text.set(fmt_float(wrapped));
                        on_change.call(wrapped);
                    }
                },
                onmouseup: move |_| drag_state.set(None),
                onmouseleave: move |_| drag_state.set(None),
            }
        }
    }
}
