use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{data_type_label, GetScriptGlobalsRequest, GetScriptGlobalsResponse};
use crate::rpc;

#[component]
pub fn ScriptView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetScriptGlobalsResponse, String>>);
    let mut search = use_signal(String::new);
    let mut mounted = use_signal(|| false);

    let mut fetch = move || {
        if loading() {
            return;
        }
        loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetScriptGlobalsRequest).await;
            data.set(Some(result));
            loading.set(false);
        });
    };

    if !mounted() {
        mounted.set(true);
        fetch();
    }

    rsx! {
        div { class: "flex flex-col flex-1 min-h-0",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                h2 { class: "text-white font-bold text-sm", "Script Globals" }
                input {
                    class: "ml-3 flex-1 px-3 py-1 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-sm",
                    placeholder: "Filter...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button {
                    class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: loading(),
                    onclick: move |_| fetch(),
                    if loading() { "Refreshing..." } else { "Refresh" }
                }
            }
            div { class: "flex-1 overflow-auto bg-gray-800 p-4 font-mono text-xs",
                match data() {
                    Some(Ok(resp)) => {
                        let query = search().to_lowercase();
                        let filtered: Vec<_> = resp.globals.iter()
                            .filter(|g| query.is_empty() || g.key.to_lowercase().contains(&query))
                            .cloned()
                            .collect();
                        let total = resp.globals.len();
                        let shown = filtered.len();
                        rsx! {
                            if total == 0 {
                                p { class: "text-gray-500", "No script globals (not in an event/script right now?)." }
                            } else {
                                p { class: "text-gray-500 mb-2",
                                    if query.is_empty() { "{total} globals" } else { "{shown} / {total} globals" }
                                }
                            }
                            for g in filtered.into_iter() {
                                div { key: "{g.key}", class: "flex items-baseline gap-2 py-0.5 border-b border-gray-700/50",
                                    span { class: "text-indigo-300 shrink-0", "{g.key}" }
                                    span { class: "text-gray-500 shrink-0", "({data_type_label(g.kind)})" }
                                    span { class: "text-white truncate", "{g.value}" }
                                }
                            }
                            if shown == 0 && total != 0 {
                                p { class: "text-gray-500 italic", "No matches" }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! { p { class: "text-red-500", "Error: {err}" } },
                    None => rsx! { p { class: "text-gray-400", "Loading globals..." } },
                }
            }
        }
    }
}
