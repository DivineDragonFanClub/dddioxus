use dioxus::prelude::*;

use crate::components::ui::{
    Button, Card, EditableText, EmptyState, ListRow, SearchField, SectionLabel, Select, StateKind,
};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetGlobalVariablesRequest, GetGlobalVariablesResponse, GlobalVariable, SetGlobalVariableRequest,
};
use crate::rpc;

#[component]
pub fn GlobalsView(temporary_only: bool) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetGlobalVariablesResponse, String>>);
    let mut mounted = use_signal(|| false);

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

    let on_commit = move |req: SetGlobalVariableRequest| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, req).await {
                data.with_mut(|slot| {
                    if let Some(Ok(response)) = slot.as_mut() {
                        if let Some(row) =
                            response.variables.iter_mut().find(|v| v.name == resp.name)
                        {
                            row.kind = resp.kind;
                            row.value = resp.value;
                        }
                    }
                });
            }
        });
    };

    rsx! {
        GlobalsPanel {
            data: data(),
            loading: loading(),
            temporary_only,
            on_refresh: move |_| fetch(),
            on_commit: on_commit,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct GlobalsPanelProps {
    pub data: Option<Result<GetGlobalVariablesResponse, String>>,
    pub loading: bool,
    pub temporary_only: bool,
    pub on_refresh: EventHandler<()>,
    pub on_commit: EventHandler<SetGlobalVariableRequest>,
}

#[component]
pub fn GlobalsPanel(props: GlobalsPanelProps) -> Element {
    let mut search = use_signal(String::new);
    // "all" | "number" | "string"
    let mut kind_filter = use_signal(|| "all".to_string());
    // "all" | "global" | "local", locked to local on the map embed
    let mut scope_filter = use_signal(|| "all".to_string());
    let on_refresh = props.on_refresh;
    let on_commit = props.on_commit;
    let title = if props.temporary_only { "Local Variables" } else { "Variables" };

    rsx! {
        Card {
            class: "flex-1",
            title,
            padded: false,
                header: rsx! {
                    SearchField {
                        value: search(),
                        class: "w-56",
                        on_input: move |v| search.set(v),
                    }
                    Select {
                        title: "Value type",
                        on_change: move |v| kind_filter.set(v),
                        option { value: "all", selected: kind_filter() == "all", "All types" }
                        option { value: "number", selected: kind_filter() == "number", "Number" }
                        option { value: "string", selected: kind_filter() == "string", "String" }
                    }
                    if !props.temporary_only {
                        Select {
                            title: "Scope",
                            on_change: move |v| scope_filter.set(v),
                            option { value: "all", selected: scope_filter() == "all", "All scopes" }
                            option { value: "global", selected: scope_filter() == "global", "Global" }
                            option { value: "local", selected: scope_filter() == "local", "Local" }
                        }
                    }
                    Button {
                        disabled: props.loading,
                        onclick: move |_| on_refresh.call(()),
                        if props.loading { "Refreshing\u{2026}" } else { "Refresh" }
                    }
                },
                div { class: "p-3",
                    match props.data.as_ref() {
                        Some(Ok(resp)) => {
                            let query = search().to_lowercase();
                            let temporary_only = props.temporary_only;
                            let kind = kind_filter();
                            let scope = scope_filter();
                            // the map embed only ever shows local vars, otherwise honor the scope dropdown
                            let pool: Vec<_> = resp.variables.iter()
                                .filter(|v| !temporary_only || v.temporary)
                                .collect();
                            let filtered: Vec<_> = pool.iter()
                                .filter(|v| match kind.as_str() {
                                    "number" => v.kind == "number",
                                    "string" => v.kind == "string",
                                    _ => true,
                                })
                                .filter(|v| match scope.as_str() {
                                    "global" => !v.temporary,
                                    "local" => v.temporary,
                                    _ => true,
                                })
                                .filter(|v| query.is_empty() || v.name.to_lowercase().contains(&query))
                                .map(|v| (*v).clone())
                                .collect();
                            let total = pool.len();
                            let shown = filtered.len();
                            let count = if shown == total {
                                format!("{total} variables")
                            } else {
                                format!("{shown} / {total} variables")
                            };
                            rsx! {
                                SectionLabel { label: "{count}", class: "mb-2" }
                                for v in filtered.into_iter() {
                                    GlobalRow {
                                        key: "{v.name}",
                                        variable: v,
                                        on_commit: on_commit,
                                    }
                                }
                                if shown == 0 {
                                    EmptyState { kind: StateKind::Empty, message: "No matches" }
                                }
                            }
                        }
                        Some(Err(err)) => rsx! {
                            EmptyState { kind: StateKind::Error, message: "Error: {err}" }
                        },
                        None => rsx! {
                            EmptyState { kind: StateKind::Loading, message: "Loading variables\u{2026}" }
                        },
                    }
                }
            }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct GlobalRowProps {
    pub variable: GlobalVariable,
    pub on_commit: EventHandler<SetGlobalVariableRequest>,
}

#[component]
pub fn GlobalRow(props: GlobalRowProps) -> Element {
    let name = props.variable.name.clone();
    let kind = props.variable.kind.clone();
    let on_commit = props.on_commit;

    let kind_class = if kind == "string" {
        "text-blue-400 w-14 text-xs shrink-0"
    } else {
        "text-emerald-400 w-14 text-xs shrink-0"
    };

    rsx! {
        ListRow {
            span { class: "{kind_class}", "{kind}" }
            span { class: "text-gray-200 flex-1 truncate font-mono text-xs", "{name}" }
            EditableText {
                value: props.variable.value.clone(),
                width: "w-40",
                on_commit: move |value: String| {
                    on_commit.call(SetGlobalVariableRequest {
                        name: name.clone(),
                        kind: kind.clone(),
                        value,
                    });
                },
            }
        }
    }
}
