use std::time::Duration;

use dioxus::prelude::*;

use super::fields::Vec3Editor;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{GetTransformRequest, GetTransformResponse, SetTransformRequest, Vec3};
use crate::rpc;

/// Watch-mode poll interval. ~60 FPS — chosen so the panel feels live
/// while the game runs. One panel ≈ 60 RPS; the user sees a pulsing
/// indicator on the header so the cost is obvious.
const WATCH_INTERVAL: Duration = Duration::from_millis(16);

#[derive(PartialEq, Clone, Props)]
pub struct TransformInspectorProps {
    path: String,
}

#[component]
pub fn TransformInspector(props: TransformInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut data = use_signal(|| None::<Result<GetTransformResponse, String>>);
    let mut loading = use_signal(|| false);
    let mut watching = use_signal(|| false);
    let path = props.path.clone();

    let mut fetch = {
        let path = path.clone();
        move || {
            if loading() {
                return;
            }
            loading.set(true);
            let path = path.clone();
            spawn(async move {
                let result = rpc::call(&conn, GetTransformRequest { path }).await;
                data.set(Some(result));
                loading.set(false);
            });
        }
    };

    let mut last_path = use_signal(String::new);
    if last_path() != path {
        last_path.set(path.clone());
        fetch();
    }

    // Watch loop: while `watching` is true, fire `GetTransformRequest`
    // every WATCH_INTERVAL. Subscribing to both `watching()` and
    // `last_path()` re-runs the effect when either changes — so
    // toggling off, or selecting a different node, immediately starts
    // a fresh task. The old task notices via `peek()` that its captured
    // `path` no longer matches `last_path`, and breaks within one tick
    // without us tracking a JoinHandle.
    use_effect(move || {
        if !watching() {
            return;
        }
        let path = last_path();
        spawn(async move {
            loop {
                if !*watching.peek() {
                    break;
                }
                if *last_path.peek() != path {
                    break;
                }
                let result = rpc::call(&conn, GetTransformRequest { path: path.clone() }).await;
                data.set(Some(result));
                tokio::time::sleep(WATCH_INTERVAL).await;
            }
        });
    });

    let on_change = move |req: SetTransformRequest| {
        spawn(async move {
            let _ = rpc::call(&conn, req).await;
        });
    };

    rsx! {
        TransformPanel {
            data: data(),
            watching: watching(),
            on_refresh: move |_| fetch(),
            on_toggle_watch: move |_| watching.toggle(),
            on_change: on_change,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct TransformPanelProps {
    pub data: Option<Result<GetTransformResponse, String>>,
    pub on_refresh: EventHandler<()>,
    pub on_change: EventHandler<SetTransformRequest>,
    /// True when watch-mode polling is active. Renders a pulsing dot in
    /// the header so the user can see the panel is hammering the server.
    #[props(default)]
    pub watching: bool,
    /// Toggle watch mode. Defaults to a no-op so storybook fixtures can
    /// stamp out static `TransformPanel`s without wiring a handler.
    #[props(default)]
    pub on_toggle_watch: EventHandler<()>,
}

#[component]
pub fn TransformPanel(props: TransformPanelProps) -> Element {
    let on_refresh = props.on_refresh;
    let on_change = props.on_change;
    let on_toggle_watch = props.on_toggle_watch;
    let watching = props.watching;
    let watch_btn_class = if watching {
        "text-red-400 hover:text-red-300 text-xs flex items-center gap-1"
    } else {
        "text-gray-400 hover:text-white text-xs flex items-center gap-1"
    };
    let watch_btn_title = if watching {
        "Stop polling — panel is hitting the server every 16 ms"
    } else {
        "Watch — poll every 16 ms (~60 FPS) until toggled off"
    };

    match props.data.as_ref() {
        Some(Ok(tf)) => {
            let tf = tf.clone();
            let p_pos = tf.path.clone();
            let p_rot = tf.path.clone();
            let p_scale = tf.path.clone();

            rsx! {
                div {
                    "data-component": "TransformPanel",
                    class: "p-3 bg-gray-900 border-t border-gray-700 font-mono text-xs",
                    div { class: "flex items-center justify-between mb-2",
                        h3 { class: "text-white font-bold text-sm", "Transform" }
                        div { class: "flex items-center gap-2",
                            button {
                                class: "{watch_btn_class}",
                                title: "{watch_btn_title}",
                                onclick: move |_| on_toggle_watch.call(()),
                                if watching {
                                    span { class: "inline-block w-2 h-2 rounded-full bg-red-500 animate-pulse" }
                                    "Watching"
                                } else {
                                    "Watch"
                                }
                            }
                            button {
                                class: "text-gray-400 hover:text-white text-xs",
                                onclick: move |_| on_refresh.call(()),
                                "↻"
                            }
                        }
                    }
                    Vec3Editor {
                        label: "Position",
                        value: tf.local_position,
                        on_change: move |v: Vec3| on_change.call(SetTransformRequest {
                            path: p_pos.clone(),
                            local_position: Some(v),
                            local_rotation: None,
                            local_scale: None,
                        }),
                    }
                    Vec3Editor {
                        label: "Rotation",
                        value: tf.local_rotation,
                        wrap: Some(360.0),
                        on_change: move |v: Vec3| on_change.call(SetTransformRequest {
                            path: p_rot.clone(),
                            local_position: None,
                            local_rotation: Some(v),
                            local_scale: None,
                        }),
                    }
                    Vec3Editor {
                        label: "Scale",
                        value: tf.local_scale,
                        on_change: move |v: Vec3| on_change.call(SetTransformRequest {
                            path: p_scale.clone(),
                            local_position: None,
                            local_rotation: None,
                            local_scale: Some(v),
                        }),
                    }
                }
            }
        }
        Some(Err(err)) => rsx! {
            div { class: "p-3 bg-gray-900 border-t border-gray-700 text-xs text-red-500",
                "Transform error: {err}"
            }
        },
        None => rsx! {
            div { class: "p-3 bg-gray-900 border-t border-gray-700 text-xs text-gray-500",
                "Loading transform..."
            }
        },
    }
}
