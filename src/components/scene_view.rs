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
        if loading() { return; }
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

    match data().as_ref() {
        Some(Ok(resp)) => rsx! {
            div { class: "flex flex-col h-full",
                div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                    button {
                        class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                        disabled: loading(),
                        onclick: move |_| fetch(),
                        if loading() { "Refreshing..." } else { "Refresh" }
                    }
                    if let Some(path) = selected_path() {
                        span { class: "text-gray-500 text-xs ml-2 truncate", "{path}" }
                    }
                }
                div { class: "flex flex-1 overflow-hidden",
                    div { class: "flex-1 overflow-auto bg-gray-800",
                        SceneTree {
                            scenes: resp.scenes.clone(),
                            selected_path: selected_path(),
                            on_select: move |path: String| selected_path.set(Some(path)),
                            on_toggle_active: toggle_active,
                        }
                    }
                    if let Some(path) = selected_path() {
                        Inspector { path }
                    }
                }
            }
        },
        Some(Err(err)) => rsx! {
            div { class: "p-4",
                p { class: "text-red-500", "Error: {err}" }
                button {
                    class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm mt-2",
                    onclick: move |_| fetch(),
                    "Retry"
                }
            }
        },
        None => rsx! {
            div { class: "p-4 text-gray-400", "Loading scene..." }
        },
    }
}
