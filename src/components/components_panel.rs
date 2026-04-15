use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    ComponentInfo, GetComponentsRequest, GetComponentsResponse, ToggleComponentRequest,
};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct ComponentsPanelProps {
    pub path: String,
}

#[component]
pub fn ComponentsPanel(props: ComponentsPanelProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut data = use_signal(|| None::<Result<GetComponentsResponse, String>>);
    let mut loading = use_signal(|| false);
    let path = props.path.clone();

    let fetch = use_callback({
        let path = path.clone();
        move |_: ()| {
            if loading() { return; }
            loading.set(true);
            let path = path.clone();
            spawn(async move {
                let result = rpc::call(&conn, GetComponentsRequest { path }).await;
                data.set(Some(result));
                loading.set(false);
            });
        }
    });

    let on_toggle = use_callback({
        let path = path.clone();
        move |index: u32| {
            let path = path.clone();
            spawn(async move {
                let _ = rpc::call(&conn, ToggleComponentRequest { path, index }).await;
                fetch.call(());
            });
        }
    });

    let mut last_path = use_signal(String::new);
    if last_path() != path {
        last_path.set(path.clone());
        fetch.call(());
    }

    match data().as_ref() {
        Some(Ok(resp)) => {
            let components = resp.components.clone();
            rsx! {
                div { class: "p-3 border-t border-gray-700 font-mono text-xs",
                    div { class: "flex items-center justify-between mb-2",
                        h3 { class: "text-white font-bold text-sm", "Components ({components.len()})" }
                        button {
                            class: "text-gray-400 hover:text-white text-xs",
                            onclick: move |_| fetch.call(()),
                            "↻"
                        }
                    }
                    for component in components.iter() {
                        ComponentRow {
                            key: "{component.index}",
                            component: component.clone(),
                            on_toggle: on_toggle,
                        }
                    }
                }
            }
        }
        Some(Err(err)) => rsx! {
            div { class: "p-3 border-t border-gray-700 text-xs text-red-500",
                "Error: {err}"
            }
        },
        None => rsx! {
            div { class: "p-3 border-t border-gray-700 text-xs text-gray-500",
                "Loading components..."
            }
        },
    }
}

#[derive(PartialEq, Clone, Props)]
struct ComponentRowProps {
    component: ComponentInfo,
    on_toggle: Callback<u32>,
}

#[component]
fn ComponentRow(props: ComponentRowProps) -> Element {
    let index = props.component.index;
    let enabled = props.component.enabled;
    let on_toggle = props.on_toggle;

    let toggle = move |_| on_toggle.call(index);

    let (icon, color) = match enabled {
        Some(true) => ("●", "text-green-400 hover:text-red-400"),
        Some(false) => ("○", "text-gray-500 hover:text-green-400"),
        None => ("—", "text-gray-600"),
    };
    let name_class = match enabled {
        Some(false) => "text-gray-500 line-through",
        _ => "text-gray-200",
    };

    rsx! {
        div { class: "flex items-center gap-2 py-0.5 hover:bg-gray-800 rounded px-1",
            if enabled.is_some() {
                button {
                    class: "{color} w-4",
                    onclick: toggle,
                    "{icon}"
                }
            } else {
                span { class: "{color} w-4", "{icon}" }
            }
            span { class: "{name_class} truncate", "{props.component.type_name}" }
        }
    }
}
