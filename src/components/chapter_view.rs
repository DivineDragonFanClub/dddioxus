use dioxus::prelude::*;

use crate::components::toast::use_toasts;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Card, Checkbox, EmptyState, ListRow, SearchField, SectionLabel,
    StateKind, Tone,
};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetChaptersRequest, GetChaptersResponse, SetChapterClearedRequest, SetCurrentChapterRequest,
};
use crate::rpc;

#[component]
pub fn ChapterView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let toasts = use_toasts();
    let mut data = use_signal(|| None::<GetChaptersResponse>);
    let mut loading = use_signal(|| false);
    let mut search = use_signal(String::new);
    let mut mounted = use_signal(|| false);

    let refresh = use_callback(move |_: ()| {
        if loading() {
            return;
        }
        loading.set(true);
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetChaptersRequest).await {
                data.set(Some(resp));
            }
            loading.set(false);
        });
    });

    if !mounted() {
        mounted.set(true);
        refresh.call(());
    }

    let set_current = move |cid: String| {
        spawn(async move {
            match rpc::call(&conn, SetCurrentChapterRequest { cid }).await {
                Ok(resp) => {
                    toasts.show(format!("Set current chapter. Reset {} later chapters.", resp.reset));
                    refresh.call(());
                }
                Err(e) => toasts.show(format!("Set current failed: {e}")),
            }
        });
    };

    let set_cleared = move |(cid, cleared): (String, bool)| {
        spawn(async move {
            match rpc::call(&conn, SetChapterClearedRequest { cid, cleared }).await {
                Ok(_) => refresh.call(()),
                Err(e) => toasts.show(format!("Change failed: {e}")),
            }
        });
    };

    rsx! {
        Card {
            class: "flex-1",
            padded: false,
            header: rsx! {
                SearchField {
                    value: search(),
                    placeholder: "Filter chapters\u{2026}",
                    class: "w-56",
                    on_input: move |v| search.set(v),
                }
                Button {
                    disabled: loading(),
                    onclick: move |_| refresh.call(()),
                    if loading() { "Refreshing\u{2026}" } else { "Refresh" }
                }
            },
            div { class: "p-3",
                match data() {
                    None => rsx! { EmptyState { kind: StateKind::Loading, message: "Loading chapters\u{2026}" } },
                    Some(resp) if !resp.available => rsx! {
                        EmptyState { kind: StateKind::Empty, message: "No save loaded. Load a save and hit Refresh." }
                    },
                    Some(resp) => {
                        let query = search().to_lowercase();
                        let current = resp.current_cid.clone();
                        let progress = resp.progress;
                        let chapters: Vec<_> = resp.chapters.into_iter()
                            .filter(|c| query.is_empty()
                                || c.title.to_lowercase().contains(&query)
                                || c.cid.to_lowercase().contains(&query)
                                || c.kind.to_lowercase().contains(&query))
                            .collect();
                        rsx! {
                            SectionLabel {
                                label: "Current {current}  \u{00B7}  progress {progress}  \u{00B7}  {chapters.len()} chapters",
                                class: "mb-2",
                            }
                            for ch in chapters.into_iter() {
                                ListRow {
                                    key: "{ch.cid}",
                                    selected: ch.is_current,
                                    Checkbox {
                                        checked: ch.cleared,
                                        title: "Cleared",
                                        on_toggle: {
                                            let cid = ch.cid.clone();
                                            move |v| set_cleared((cid.clone(), v))
                                        },
                                    }
                                    span {
                                        class: "text-gray-500 text-xs w-16 shrink-0 truncate",
                                        title: "{ch.kind}",
                                        "{ch.kind}"
                                    }
                                    div { class: "flex-1 min-w-0",
                                        p { class: "text-white truncate", title: "{ch.cid}",
                                            "{ch.title}"
                                            if ch.is_current {
                                                span { class: "ml-2 text-indigo-300 text-xs", "current" }
                                            }
                                        }
                                        p { class: "text-gray-500 font-mono text-xs truncate", "{ch.cid}" }
                                    }
                                    if ch.story {
                                        Button {
                                            tone: Tone::Indigo,
                                            variant: ButtonVariant::Outline,
                                            size: ButtonSize::Sm,
                                            disabled: ch.is_current,
                                            title: "Make this the current chapter and reset it plus every later story chapter to not-complete. Earlier chapters are left as-is",
                                            onclick: {
                                                let cid = ch.cid.clone();
                                                move |_| set_current(cid.clone())
                                            },
                                            "Set current"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
