use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetMessLabelsRequest, GetMessLabelsResponse, ListMessFilesRequest, ListMessFilesResponse,
    MessEntry, MessFileInfo,
};
use crate::rpc;

const PANEL_MIN_WIDTH: f64 = 220.0;
const PANEL_MAX_WIDTH: f64 = 600.0;
const PANEL_DEFAULT_WIDTH: f64 = 320.0;

#[component]
pub fn MessView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut files_loading = use_signal(|| false);
    let mut files = use_signal(|| None::<Result<ListMessFilesResponse, String>>);
    let mut mounted = use_signal(|| false);

    let mut selected_file = use_signal(|| None::<String>);
    let mut last_loaded_file = use_signal(String::new);
    let mut entries = use_signal(|| None::<Result<GetMessLabelsResponse, String>>);
    let mut entries_loading = use_signal(|| false);

    let mut fetch_files = move || {
        if files_loading() {
            return;
        }
        files_loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, ListMessFilesRequest).await;
            files.set(Some(result));
            files_loading.set(false);
        });
    };

    if !mounted() {
        mounted.set(true);
        fetch_files();
    }

    let current_file = selected_file();
    let current_key = current_file.clone().unwrap_or_default();
    if last_loaded_file() != current_key {
        last_loaded_file.set(current_key.clone());
        match current_file {
            Some(name) => {
                entries.set(None);
                entries_loading.set(true);
                spawn(async move {
                    let req = GetMessLabelsRequest { file: name };
                    let result = rpc::call(&conn, req).await;
                    entries.set(Some(result));
                    entries_loading.set(false);
                });
            }
            None => entries.set(None),
        }
    }

    let on_select_file = use_callback(move |name: String| selected_file.set(Some(name)));

    rsx! {
        MessPanel {
            files: files(),
            files_loading: files_loading(),
            selected: selected_file(),
            entries: entries(),
            entries_loading: entries_loading(),
            on_refresh_files: move |_| fetch_files(),
            on_select_file: on_select_file,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct MessPanelProps {
    pub files: Option<Result<ListMessFilesResponse, String>>,
    pub files_loading: bool,
    pub selected: Option<String>,
    pub entries: Option<Result<GetMessLabelsResponse, String>>,
    pub entries_loading: bool,
    pub on_refresh_files: EventHandler<()>,
    pub on_select_file: Callback<String>,
}

#[component]
pub fn MessPanel(props: MessPanelProps) -> Element {
    let mut panel_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut drag_state = use_signal(|| None::<DragState>);
    let on_refresh_files = props.on_refresh_files;
    let on_select_file = props.on_select_file;
    let dragging = drag_state().is_some();

    rsx! {
        div {
            "data-component": "MessPanel",
            class: "flex flex-col h-full relative",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                h2 { class: "text-white font-bold text-sm", "Mess Browser" }
                button {
                    class: "ml-auto text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: props.files_loading,
                    onclick: move |_| on_refresh_files.call(()),
                    if props.files_loading { "Refreshing..." } else { "Refresh" }
                }
            }
            div { class: "flex flex-1 overflow-hidden",
                div {
                    class: "shrink-0 bg-gray-900 border-r border-gray-700 overflow-auto",
                    style: "width: {panel_width()}px;",
                    FileList {
                        files: props.files.clone(),
                        selected: props.selected.clone(),
                        on_select: on_select_file,
                    }
                }
                div {
                    class: "w-1 bg-gray-700 hover:bg-indigo-500 cursor-col-resize",
                    onmousedown: move |e| {
                        let coord = e.client_coordinates();
                        drag_state.set(Some(DragState {
                            start_x: coord.x,
                            start_width: panel_width(),
                        }));
                    },
                }
                div { class: "flex-1 overflow-auto bg-gray-800 p-4 font-mono text-xs",
                    EntryList {
                        selected: props.selected.clone(),
                        entries: props.entries.clone(),
                        loading: props.entries_loading,
                    }
                }
            }
            if dragging {
                div {
                    class: "fixed inset-0 z-50 cursor-col-resize",
                    onmousemove: move |e| {
                        if let Some(state) = drag_state() {
                            let coord = e.client_coordinates();
                            let delta = coord.x - state.start_x;
                            let next = (state.start_width + delta).clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);
                            panel_width.set(next);
                        }
                    },
                    onmouseup: move |_| drag_state.set(None),
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct DragState {
    start_x: f64,
    start_width: f64,
}

#[derive(PartialEq, Clone, Props)]
struct FileListProps {
    files: Option<Result<ListMessFilesResponse, String>>,
    selected: Option<String>,
    on_select: Callback<String>,
}

#[component]
fn FileList(props: FileListProps) -> Element {
    let mut search = use_signal(String::new);
    let on_select = props.on_select;

    rsx! {
        div { class: "p-2",
            input {
                class: "w-full mb-2 px-2 py-1 bg-gray-800 text-white rounded border border-gray-700 focus:border-indigo-500 focus:outline-none text-xs",
                placeholder: "Filter files...",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }
            match props.files.as_ref() {
                Some(Ok(resp)) => {
                    let query = search().to_lowercase();
                    let filtered: Vec<MessFileInfo> = resp.files.iter()
                        .filter(|f| query.is_empty() || f.name.to_lowercase().contains(&query))
                        .cloned()
                        .collect();
                    let total = resp.files.len();
                    let shown = filtered.len();
                    let selected = props.selected.clone();
                    rsx! {
                        p { class: "text-gray-500 text-[10px] mb-1 px-1",
                            if query.is_empty() { "{total} files" }
                            else { "{shown} / {total}" }
                        }
                        for f in filtered.into_iter() {
                            FileRow {
                                key: "{f.name}",
                                file: f.clone(),
                                active: selected.as_deref() == Some(f.name.as_str()),
                                on_select: on_select,
                            }
                        }
                        if shown == 0 {
                            p { class: "text-gray-500 italic text-xs px-1", "No matches" }
                        }
                    }
                }
                Some(Err(err)) => rsx! {
                    p { class: "text-red-500 text-xs", "Error: {err}" }
                },
                None => rsx! {
                    p { class: "text-gray-400 text-xs", "Loading..." }
                },
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct FileRowProps {
    file: MessFileInfo,
    active: bool,
    on_select: Callback<String>,
}

#[component]
fn FileRow(props: FileRowProps) -> Element {
    let on_select = props.on_select;
    let name = props.file.name.clone();
    let class = if props.active {
        "block w-full text-left px-2 py-1 rounded text-xs bg-gray-700 text-white"
    } else {
        "block w-full text-left px-2 py-1 rounded text-xs text-gray-300 hover:bg-gray-800"
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| on_select.call(name.clone()),
            div { class: "truncate", "{props.file.name}" }
            div { class: "text-[10px] text-gray-500",
                "{props.file.label_count} labels • refs {props.file.reference_count}"
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct EntryListProps {
    selected: Option<String>,
    entries: Option<Result<GetMessLabelsResponse, String>>,
    loading: bool,
}

#[component]
fn EntryList(props: EntryListProps) -> Element {
    let mut search = use_signal(String::new);

    let Some(name) = props.selected.as_ref() else {
        return rsx! {
            p { class: "text-gray-500 italic", "Select a file from the list." }
        };
    };

    if props.loading {
        return rsx! {
            p { class: "text-gray-400", "Loading entries from {name}..." }
        };
    }

    rsx! {
        div { class: "flex items-center gap-2 mb-3",
            input {
                class: "flex-1 px-2 py-1 bg-gray-900 text-white rounded border border-gray-700 focus:border-indigo-500 focus:outline-none text-xs",
                placeholder: "Filter labels or text...",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }
        }
        match props.entries.as_ref() {
            Some(Ok(resp)) => {
                let query = search().to_lowercase();
                let filtered: Vec<MessEntry> = resp.entries.iter()
                    .filter(|e| query.is_empty()
                        || e.label.to_lowercase().contains(&query)
                        || e.text.to_lowercase().contains(&query))
                    .cloned()
                    .collect();
                let total = resp.entries.len();
                let shown = filtered.len();
                rsx! {
                    p { class: "text-gray-500 mb-2",
                        if query.is_empty() { "{total} entries" }
                        else { "{shown} / {total}" }
                    }
                    for e in filtered.into_iter() {
                        EntryRow { key: "{e.label}", entry: e }
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
                p { class: "text-gray-400", "Loading..." }
            },
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct EntryRowProps {
    entry: MessEntry,
}

#[component]
fn EntryRow(props: EntryRowProps) -> Element {
    rsx! {
        div { class: "py-1 border-b border-gray-700/50",
            div { class: "text-blue-400 truncate", "{props.entry.label}" }
            div { class: "text-yellow-200 whitespace-pre-wrap", "{props.entry.text}" }
        }
    }
}
