use dioxus::prelude::*;

use super::inspector::Inspector;
use super::scene_tree::SceneTree;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{GetSceneNameRequest, GetSceneNameResponse, ToggleGameObjectRequest};
use crate::rpc;

#[component]
pub fn SceneView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetSceneNameResponse, String>>);
    let mut selected_path = use_signal(|| None::<String>);
    let mut mounted = use_signal(|| false);

    let mut fetch = move || {
        if loading() {
            return;
        }
        loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetSceneNameRequest).await;
            data.set(Some(result));
            loading.set(false);
        });
    };

    if !mounted() {
        mounted.set(true);
        fetch();
    }

    let toggle_active = move |path: String| {
        spawn(async move {
            if rpc::call(&conn, ToggleGameObjectRequest { path }).await.is_ok() {
                let result = rpc::call(&conn, GetSceneNameRequest).await;
                data.set(Some(result));
            }
        });
    };

    rsx! {
        div { class: "flex flex-1 h-full",
            ScenePanel {
                data: data(),
                loading: loading(),
                selected_path: selected_path(),
                on_refresh: move |_| fetch(),
                on_select: move |path: String| selected_path.set(Some(path)),
                on_toggle_active: toggle_active,
            }
            match selected_path() {
                Some(path) => rsx! { Inspector { path } },
                None => rsx! {
                    div { class: "flex flex-col shrink-0 bg-gray-900 border-l border-gray-700 overflow-y-auto",
                        div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                            h3 { class: "text-white font-bold text-sm", "Inspector" }
                            p { class: "text-gray-500 text-xs", "No selection" }
                        }
                        p { class: "p-4 text-gray-500 text-xs italic",
                            "Select a node in the scene tree to inspect its transform and components."
                        }
                    }
                },
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ScenePanelProps {
    pub data: Option<Result<GetSceneNameResponse, String>>,
    pub loading: bool,
    pub selected_path: Option<String>,
    pub on_refresh: EventHandler<()>,
    pub on_select: EventHandler<String>,
    pub on_toggle_active: EventHandler<String>,
}

#[component]
pub fn ScenePanel(props: ScenePanelProps) -> Element {
    let on_refresh = props.on_refresh;

    rsx! {
        div {
            "data-component": "ScenePanel",
            class: "flex flex-col flex-1 overflow-hidden",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                button {
                    class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: props.loading,
                    onclick: move |_| on_refresh.call(()),
                    if props.loading { "Refreshing..." } else { "Refresh" }
                }
                if let Some(path) = props.selected_path.as_ref() {
                    span { class: "text-gray-500 text-xs ml-2 truncate", "{path}" }
                }
            }
            div { class: "flex-1 overflow-auto bg-gray-800",
                match props.data.as_ref() {
                    Some(Ok(resp)) => rsx! {
                        SceneTree {
                            scenes: resp.scenes.clone(),
                            selected_path: props.selected_path.clone(),
                            on_select: props.on_select,
                            on_toggle_active: props.on_toggle_active,
                        }
                    },
                    Some(Err(err)) => rsx! {
                        div { class: "p-4",
                            p { class: "text-red-500", "Error: {err}" }
                            button {
                                class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm mt-2",
                                onclick: move |_| on_refresh.call(()),
                                "Retry"
                            }
                        }
                    },
                    None => rsx! {
                        div { class: "p-4 text-gray-400", "Loading scene..." }
                    },
                }
            }
        }
    }
}
