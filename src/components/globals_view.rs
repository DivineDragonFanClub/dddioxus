use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetGlobalVariablesRequest, GetGlobalVariablesResponse, GlobalVariable, SetGlobalVariableRequest,
};
use crate::rpc;

#[component]
pub fn GlobalsView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetGlobalVariablesResponse, String>>);
    let mut mounted = use_signal(|| false);
    let mut search = use_signal(String::new);

    let mut fetch = move || {
        if loading() { return; }
        loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetGlobalVariablesRequest).await;
            data.set(Some(result));
            loading.set(false);
        });
    };

    if !mounted() {
        mounted.set(true);
        fetch();
    }

    let apply = use_callback(move |updated: GlobalVariable| {
        data.with_mut(|slot| {
            if let Some(Ok(resp)) = slot.as_mut() {
                if let Some(row) = resp.variables.iter_mut().find(|v| v.name == updated.name) {
                    row.kind = updated.kind;
                    row.value = updated.value;
                }
            }
        });
    });

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                h2 { class: "text-white font-bold text-sm", "Global Variables" }
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
                match data().as_ref() {
                    Some(Ok(resp)) => {
                        let query = search().to_lowercase();
                        let filtered: Vec<_> = resp.variables.iter()
                            .filter(|v| query.is_empty() || v.name.to_lowercase().contains(&query))
                            .cloned()
                            .collect();
                        let total = resp.variables.len();
                        let shown = filtered.len();
                        rsx! {
                            p { class: "text-gray-500 mb-2",
                                if query.is_empty() { "{total} variables" }
                                else { "{shown} / {total} variables" }
                            }
                            for v in filtered.into_iter() {
                                GlobalRow {
                                    key: "{v.name}",
                                    variable: v,
                                    on_change: move |updated| apply.call(updated),
                                }
                            }
                            if shown == 0 {
                                p { class: "text-gray-500 italic", "No matches" }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! {
                        p { class: "text-red-500", "Error: {err}" }
                    },
                    None => rsx! {
                        p { class: "text-gray-400", "Loading variables..." }
                    },
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct GlobalRowProps {
    variable: GlobalVariable,
    on_change: EventHandler<GlobalVariable>,
}

#[component]
fn GlobalRow(props: GlobalRowProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let name = props.variable.name.clone();
    let kind = props.variable.kind.clone();
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.variable.value.clone());

    let commit = {
        let name = name.clone();
        let kind = kind.clone();
        let on_change = props.on_change;
        let current = props.variable.value.clone();
        move || {
            editing.set(false);
            if draft() == current {
                return;
            }
            let req = SetGlobalVariableRequest {
                name: name.clone(),
                kind: kind.clone(),
                value: draft(),
            };
            spawn(async move {
                if let Ok(resp) = rpc::call(&conn, req).await {
                    on_change.call(GlobalVariable {
                        name: resp.name,
                        kind: resp.kind,
                        value: resp.value,
                    });
                }
            });
        }
    };

    let kind_class = if kind == "string" {
        "text-blue-400 w-14 text-xs"
    } else {
        "text-green-400 w-14 text-xs"
    };
    let input_type = if kind == "string" { "text" } else { "number" };

    rsx! {
        div { class: "flex items-center gap-3 py-1 hover:bg-gray-700 rounded px-2",
            span { class: "{kind_class}", "{kind}" }
            span { class: "text-gray-200 flex-1 truncate", "{name}" }
            if editing() {
                input {
                    r#type: "{input_type}",
                    class: "w-40 px-2 py-0.5 bg-gray-900 text-yellow-300 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none",
                    value: "{draft}",
                    autofocus: true,
                    oninput: move |e| draft.set(e.value()),
                    onblur: {
                        let mut commit = commit.clone();
                        move |_| commit()
                    },
                    onkeydown: {
                        let mut commit = commit.clone();
                        move |e| {
                            if e.key() == Key::Enter { commit(); }
                            else if e.key() == Key::Escape { editing.set(false); }
                        }
                    },
                }
            } else {
                span {
                    class: "w-40 px-2 py-0.5 text-yellow-300 truncate cursor-text hover:bg-gray-900 rounded",
                    onclick: {
                        let value = props.variable.value.clone();
                        move |_| {
                            draft.set(value.clone());
                            editing.set(true);
                        }
                    },
                    "{props.variable.value}"
                }
            }
        }
    }
}
