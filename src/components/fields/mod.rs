use dioxus::prelude::*;
use dioxus_elements::input_data::MouseButton;

pub mod vec3;
pub use vec3::Vec3Editor;

const DEFAULT_DRAG_SENSITIVITY: f32 = 0.03;
const COARSE_MULTIPLIER: f32 = 10.0;
const FINE_MULTIPLIER: f32 = 0.1;

pub(crate) fn fmt_float(v: f32) -> String {
    format!("{:.4}", v)
}

pub(crate) fn apply_wrap(v: f32, wrap: Option<f32>) -> f32 {
    match wrap {
        Some(w) if w > 0.0 => ((v % w) + w) % w,
        _ => v,
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DragFloatProps {
    pub value: f32,
    pub on_change: EventHandler<f32>,
    #[props(default = "")]
    pub label: &'static str,
    #[props(default = "text-gray-400")]
    pub color: &'static str,
    // tailwind width of the input. defaults to a fixed width, pass "w-full" to fill a flex cell
    #[props(default = "w-20")]
    pub width: &'static str,
    #[props(default = DEFAULT_DRAG_SENSITIVITY)]
    pub sensitivity: f32,
    #[props(default)]
    pub wrap: Option<f32>,
}

#[component]
pub fn DragFloat(props: DragFloatProps) -> Element {
    let wrap = props.wrap;
    let mut text = use_signal(|| fmt_float(apply_wrap(props.value, wrap)));
    let mut last_value = use_signal(|| props.value);
    let mut drag_state = use_signal(|| None::<(f64, f32)>);

    // Skip incoming `value` updates while the user is dragging this field.
    // Without this guard, watch-mode polling (which streams server values
    // every frame) would yank the visible text away from the user's drag.
    // `peek()` deliberately avoids subscribing — we only need a snapshot.
    if drag_state.peek().is_none()
        && (*last_value.read() - props.value).abs() > f32::EPSILON
    {
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
    let input_class = format!(
        "{} px-1 py-0.5 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center min-w-0",
        props.width
    );

    rsx! {
        if !props.label.is_empty() {
            span {
                class: "{props.color} text-xs font-bold w-3 select-none",
                style: "cursor: ew-resize; box-sizing: content-box; padding: 6px 8px; margin: -6px -8px; text-align: center;",
                title: "Drag to change · Shift: coarse · Ctrl/⌘: fine",
                onmousedown: move |e: Event<MouseData>| {
                    e.prevent_default();
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
                    if !e.held_buttons().contains(MouseButton::Primary) {
                        drag_state.set(None);
                        return;
                    }
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
            }
        }
    }
}
