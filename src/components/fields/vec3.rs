use std::rc::Rc;

use arboard::Clipboard;
use dioxus::prelude::*;

use super::DragFloat;
use crate::protocol::Vec3;

/// Serialize a Vec3 as JSON and write it to the OS clipboard.
/// Format: `{"x":1.0,"y":2.0,"z":3.0}` — human-readable and
/// shareable with anything that speaks JSON.
fn copy_vec3_to_os_clipboard(v: Vec3) -> bool {
    let Ok(json) = serde_json::to_string(&v) else {
        return false;
    };
    match Clipboard::new() {
        Ok(mut clip) => clip.set_text(json).is_ok(),
        Err(_) => false,
    }
}

/// Try to read a Vec3 from the OS clipboard. Accepts anything that
/// deserializes into `Vec3` (the JSON shape written above).
fn paste_vec3_from_os_clipboard() -> Option<Vec3> {
    let mut clip = Clipboard::new().ok()?;
    let text = clip.get_text().ok()?;
    serde_json::from_str::<Vec3>(text.trim()).ok()
}

#[derive(PartialEq, Clone, Props)]
pub struct Vec3EditorProps {
    pub label: &'static str,
    pub value: Vec3,
    pub on_change: EventHandler<Vec3>,
    /// Wrap each component into `[0, w)`. Useful for rotation (360).
    #[props(default)]
    pub wrap: Option<f32>,
}

#[component]
pub fn Vec3Editor(props: Vec3EditorProps) -> Element {
    // Track each component locally so fire-and-forget parents don't
    // cause a snapback: after the user drags X to 0.5, cur_x holds
    // 0.5 even if props.value hasn't been refreshed from the server.
    let mut cur_x = use_signal(|| props.value.x);
    let mut cur_y = use_signal(|| props.value.y);
    let mut cur_z = use_signal(|| props.value.z);
    let mut last_value = use_signal(|| props.value);
    let mut menu_open = use_signal(|| false);
    // Viewport coordinates where the dropdown should anchor. We use
    // `position: fixed` for the dropdown so it escapes ancestor
    // `overflow: hidden` / `overflow: auto` clipping (StorySection,
    // Inspector sidebar, etc.).
    let mut menu_pos = use_signal(|| (0.0_f64, 0.0_f64));
    // DOM handle to the dots button, captured at mount. We use it to
    // read the button's bounding rect on click so the menu anchors to
    // the button itself regardless of where inside it the user clicked.
    let mut dots_ref = use_signal(|| None::<Rc<MountedData>>);
    // Cached result of "is there a pasteable Vec3 in the OS clipboard?",
    // refreshed when the menu is opened so we can grey out Paste.
    let mut pasteable = use_signal(|| None::<Vec3>);

    if *last_value.read() != props.value {
        last_value.set(props.value);
        cur_x.set(props.value.x);
        cur_y.set(props.value.y);
        cur_z.set(props.value.z);
    }

    let on_change = props.on_change;
    let fire = move |next: Vec3| on_change.call(next);

    let wrap = props.wrap;
    let can_paste = pasteable.read().is_some();

    rsx! {
        div {
            "data-component": "Vec3Editor",
            class: "flex items-center gap-2 mb-1",
            span { class: "text-gray-400 w-16", "{props.label}" }
            DragFloat {
                label: "X",
                color: "text-red-400",
                value: cur_x(),
                wrap,
                on_change: move |v: f32| {
                    cur_x.set(v);
                    fire(Vec3 { x: v, y: cur_y(), z: cur_z() });
                },
            }
            DragFloat {
                label: "Y",
                color: "text-green-400",
                value: cur_y(),
                wrap,
                on_change: move |v: f32| {
                    cur_y.set(v);
                    fire(Vec3 { x: cur_x(), y: v, z: cur_z() });
                },
            }
            DragFloat {
                label: "Z",
                color: "text-blue-400",
                value: cur_z(),
                wrap,
                on_change: move |v: f32| {
                    cur_z.set(v);
                    fire(Vec3 { x: cur_x(), y: cur_y(), z: v });
                },
            }
            button {
                class: "ml-1 px-1 text-gray-500 hover:text-white text-sm leading-none select-none",
                onmounted: move |e: Event<MountedData>| {
                    dots_ref.set(Some(e.data()));
                },
                onclick: move |_| {
                    if menu_open() {
                        menu_open.set(false);
                        return;
                    }
                    pasteable.set(paste_vec3_from_os_clipboard());
                    // Anchor the dropdown to the dots button's bounding
                    // rect and clamp it inside the viewport. Prefer
                    // right-of-anchor, flip to left if we'd overflow the
                    // right edge, shift up if we'd overflow the bottom.
                    if let Some(elem) = dots_ref() {
                        spawn(async move {
                            let Ok(rect) = elem.get_client_rect().await else {
                                return;
                            };
                            let mut eval = document::eval(
                                "dioxus.send([window.innerWidth, window.innerHeight])",
                            );
                            let (vw, vh) = eval
                                .recv::<(f64, f64)>()
                                .await
                                .unwrap_or((f64::MAX, f64::MAX));

                            // Estimated menu size — min-width 96 and
                            // ~2-3 items at ~24px each.
                            const MENU_W: f64 = 96.0;
                            const MENU_H: f64 = 72.0;
                            const GAP: f64 = 4.0;

                            let preferred = rect.origin.x + rect.size.width + GAP;
                            let left = if preferred + MENU_W <= vw {
                                preferred
                            } else {
                                let flipped = rect.origin.x - MENU_W - GAP;
                                flipped.max(0.0)
                            };

                            let mut top = rect.origin.y;
                            if top + MENU_H > vh {
                                top = (vh - MENU_H).max(0.0);
                            }
                            if top < 0.0 {
                                top = 0.0;
                            }

                            menu_pos.set((left, top));
                            menu_open.set(true);
                        });
                    }
                },
                "⋮"
            }
            if menu_open() {
                // Full-viewport invisible overlay — catches any click
                // outside the dropdown and closes the menu.
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| menu_open.set(false),
                }
                div {
                    class: "bg-gray-900 border border-gray-700 rounded shadow-lg py-1 text-xs",
                    style: "position: fixed; left: {menu_pos().0}px; top: {menu_pos().1}px; z-index: 50; min-width: 96px;",
                    button {
                        class: "block w-full text-left px-3 py-1 text-gray-200 hover:bg-gray-800",
                        onclick: move |_| {
                            let v = Vec3 { x: cur_x(), y: cur_y(), z: cur_z() };
                            copy_vec3_to_os_clipboard(v);
                            menu_open.set(false);
                        },
                        "Copy"
                    }
                    if can_paste {
                        button {
                            class: "block w-full text-left px-3 py-1 text-gray-200 hover:bg-gray-800",
                            onclick: move |_| {
                                if let Some(v) = pasteable() {
                                    cur_x.set(v.x);
                                    cur_y.set(v.y);
                                    cur_z.set(v.z);
                                    fire(v);
                                }
                                menu_open.set(false);
                            },
                            "Paste"
                        }
                    } else {
                        span {
                            class: "block w-full text-left px-3 py-1 text-gray-600",
                            style: "cursor: not-allowed",
                            "Paste"
                        }
                    }
                }
            }
        }
    }
}
