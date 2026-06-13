use dioxus::prelude::*;

use crate::components::toast::use_toasts;
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
        div {
            "data-component": "ChapterView",
            class: "flex flex-col flex-1 min-h-0",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700 shrink-0",
                h2 { class: "text-white font-bold text-sm shrink-0", "Chapter" }
                input {
                    class: "ml-3 flex-1 px-3 py-1 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-sm",
                    placeholder: "Filter chapters...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button {
                    class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: loading(),
                    onclick: move |_| refresh.call(()),
                    if loading() { "Refreshing..." } else { "Refresh" }
                }
            }
            div { class: "flex-1 overflow-auto bg-gray-800 p-4 text-sm",
                match data() {
                    None => rsx! { p { class: "text-gray-400", "Loading chapters..." } },
                    Some(resp) if !resp.available => rsx! {
                        p { class: "text-gray-500", "No save loaded. Load a save and hit Refresh." }
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
                            p { class: "text-gray-500 mb-2 font-mono text-xs",
                                "Current: {current}  ·  progress {progress}  ·  {chapters.len()} chapters"
                            }
                            for ch in chapters.into_iter() {
                                div {
                                    key: "{ch.cid}",
                                    class: if ch.is_current {
                                        "flex items-center gap-3 py-1.5 px-2 rounded bg-indigo-900/40 border border-indigo-700"
                                    } else {
                                        "flex items-center gap-3 py-1.5 px-2 rounded hover:bg-gray-700"
                                    },
                                    label { class: "flex items-center gap-1.5 shrink-0 cursor-pointer", title: "Cleared",
                                        input {
                                            r#type: "checkbox",
                                            checked: ch.cleared,
                                            onchange: {
                                                let cid = ch.cid.clone();
                                                move |e: Event<FormData>| set_cleared((cid.clone(), e.checked()))
                                            },
                                        }
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
                                        button {
                                            class: "text-indigo-300 hover:text-indigo-200 text-xs border border-indigo-500/40 rounded px-2 py-0.5 shrink-0",
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
