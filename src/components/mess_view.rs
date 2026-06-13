use dioxus::prelude::*;

use crate::components::resizable_panel::{ResizablePanel, Side};
use crate::components::toast::use_toasts;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Card, EmptyState, ListRow, Page, PanelHeader, SearchField,
    SectionLabel, StateKind, Tone,
};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetMessLabelsRequest, GetMessLabelsResponse, ListMessFilesRequest, ListMessFilesResponse,
    MessEntry, MessFileInfo, SetMessTextRequest,
};
use crate::rpc;

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
    let on_refresh_files = props.on_refresh_files;
    let on_select_file = props.on_select_file;

    rsx! {
        Page { row: true,
            ResizablePanel {
                side: Side::Left,
                class: "bg-gray-800/80 border border-gray-700/70 rounded-xl shadow-lg shadow-black/30 overflow-hidden",
                default_width: 320.0,
                min_width: 220.0,
                max_width: 600.0,
                // card header: title + refresh button
                div { class: "flex items-center gap-2 px-3 py-2 border-b border-gray-700/70 bg-gray-900/40 shrink-0",
                    h3 { class: "text-white font-semibold text-sm truncate", "Mess Browser" }
                    div { class: "ml-auto flex items-center gap-2",
                        Button {
                            disabled: props.files_loading,
                            onclick: move |_| on_refresh_files.call(()),
                            if props.files_loading { "Refreshing\u{2026}" } else { "Refresh" }
                        }
                    }
                }
                // file list body
                FileList {
                    files: props.files.clone(),
                    selected: props.selected.clone(),
                    on_select: on_select_file,
                }
            }
            // right pane: entry viewer
            Card {
                class: "flex-1",
                padded: false,
                EntryList {
                    selected: props.selected.clone(),
                    entries: props.entries.clone(),
                    loading: props.entries_loading,
                }
            }
        }
    }
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
        div { class: "flex flex-col min-h-0 flex-1 overflow-hidden",
            div { class: "p-2 shrink-0",
                SearchField {
                    value: search(),
                    placeholder: "Filter files\u{2026}",
                    on_input: move |v| search.set(v),
                }
            }
            div { class: "flex-1 overflow-auto px-2 pb-2",
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
                            SectionLabel {
                                label: if query.is_empty() { format!("{total} files") } else { format!("{shown} / {total}") },
                                class: "mb-1 px-1",
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
                                EmptyState { kind: StateKind::Empty, message: "No matches", compact: true }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! {
                        EmptyState { kind: StateKind::Error, message: "Error: {err}" }
                    },
                    None => rsx! {
                        EmptyState { kind: StateKind::Loading, message: "Loading\u{2026}" }
                    },
                }
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

    rsx! {
        ListRow {
            selected: props.active,
            onclick: move |_| on_select.call(name.clone()),
            div { class: "flex-1 min-w-0",
                p { class: "text-white text-xs truncate", "{props.file.name}" }
                p { class: "text-gray-500 text-[10px]",
                    "{props.file.label_count} labels \u{00B7} refs {props.file.reference_count}"
                }
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
            div { class: "flex-1 flex items-center justify-center p-6",
                EmptyState { kind: StateKind::Empty, message: "Select a file from the list." }
            }
        };
    };

    if props.loading {
        return rsx! {
            div { class: "flex-1 p-6",
                EmptyState { kind: StateKind::Loading, message: "Loading entries from {name}\u{2026}" }
            }
        };
    }

    rsx! {
        div { class: "flex flex-col min-h-0 flex-1 overflow-hidden",
            // search toolbar inside the right pane header
            PanelHeader {
                title: name.clone(),
                actions: rsx! {
                    SearchField {
                        value: search(),
                        placeholder: "Filter labels or text\u{2026}",
                        class: "w-52",
                        on_input: move |v| search.set(v),
                    }
                },
            }
            div { class: "flex-1 overflow-auto p-3 font-mono text-xs",
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
                            SectionLabel {
                                label: if query.is_empty() { format!("{total} entries") } else { format!("{shown} / {total}") },
                                class: "mb-2",
                            }
                            for e in filtered.into_iter() {
                                EntryRow { key: "{e.label}", entry: e }
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
                        EmptyState { kind: StateKind::Loading, message: "Loading\u{2026}" }
                    },
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct EntryRowProps {
    entry: MessEntry,
}

#[component]
fn EntryRow(props: EntryRowProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let toasts = use_toasts();
    let mut editing = use_signal(|| false);
    // current shows the live value, draft is what's being typed
    let mut current = use_signal(|| props.entry.text.clone());
    let mut draft = use_signal(|| props.entry.text.clone());
    let mut saving = use_signal(|| false);

    let label = props.entry.label.clone();
    let save = move |_| {
        if saving() {
            return;
        }
        saving.set(true);
        let label = label.clone();
        let text = draft();
        spawn(async move {
            match rpc::call(&conn, SetMessTextRequest { label, text }).await {
                Ok(resp) => {
                    current.set(resp.text);
                    editing.set(false);
                    toasts.show("Text saved.");
                }
                Err(e) => toasts.show(format!("Save failed: {e}")),
            }
            saving.set(false);
        });
    };

    rsx! {
        div { class: "py-1.5 border-b border-gray-700/50",
            div { class: "flex items-center gap-2",
                div { class: "text-blue-400 truncate flex-1", "{props.entry.label}" }
                if !editing() {
                    Button {
                        tone: Tone::Gray,
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            draft.set(current());
                            editing.set(true);
                        },
                        "edit"
                    }
                }
            }
            if editing() {
                textarea {
                    class: "w-full mt-1 px-2 py-1 bg-gray-950 text-amber-200 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-xs",
                    rows: "3",
                    value: "{draft}",
                    oninput: move |e| draft.set(e.value()),
                }
                div { class: "flex gap-2 mt-1",
                    Button {
                        tone: Tone::Emerald,
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        disabled: saving(),
                        onclick: save,
                        if saving() { "Saving\u{2026}" } else { "Save" }
                    }
                    Button {
                        tone: Tone::Gray,
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| editing.set(false),
                        "Cancel"
                    }
                }
            } else {
                div { class: "text-amber-200 whitespace-pre-wrap mt-0.5", "{current}" }
            }
        }
    }
}
