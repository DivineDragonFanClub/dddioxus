use dioxus::prelude::*;

use super::fields::Vec3Editor;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{GetTransformRequest, GetTransformResponse, SetTransformRequest, Vec3};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct TransformInspectorProps {
    path: String,
}

#[component]
pub fn TransformInspector(props: TransformInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut data = use_signal(|| None::<Result<GetTransformResponse, String>>);
    let mut loading = use_signal(|| false);
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

    let on_change = move |req: SetTransformRequest| {
        spawn(async move {
            let _ = rpc::call(&conn, req).await;
        });
    };

    rsx! {
        TransformPanel {
            data: data(),
            on_refresh: move |_| fetch(),
            on_change: on_change,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct TransformPanelProps {
    pub data: Option<Result<GetTransformResponse, String>>,
    pub on_refresh: EventHandler<()>,
    pub on_change: EventHandler<SetTransformRequest>,
}

#[component]
pub fn TransformPanel(props: TransformPanelProps) -> Element {
    let on_refresh = props.on_refresh;
    let on_change = props.on_change;

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
                        button {
                            class: "text-gray-400 hover:text-white text-xs",
                            onclick: move |_| on_refresh.call(()),
                            "↻"
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
