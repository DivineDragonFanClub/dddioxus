use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{BondHolderInfo, GetBondHoldersRequest, GetBondHoldersResponse};
use crate::rpc;

#[component]
pub fn BondsView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetBondHoldersResponse, String>>);
    let mut search = use_signal(String::new);
    let mut mounted = use_signal(|| false);

    let mut fetch = move || {
        if loading() {
            return;
        }
        loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetBondHoldersRequest).await;
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
                h2 { class: "text-white font-bold text-sm", "Bond Holders" }
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
            div { class: "flex-1 overflow-auto bg-gray-800 p-4 text-xs",
                match data() {
                    Some(Ok(resp)) => {
                        let query = search().to_lowercase();
                        let filtered: Vec<_> = resp.holders.iter()
                            .filter(|h| query.is_empty() || h.name.to_lowercase().contains(&query) || h.gid.to_lowercase().contains(&query))
                            .cloned()
                            .collect();
                        let total = resp.holders.len();
                        rsx! {
                            if total == 0 {
                                p { class: "text-gray-500", "No bond holders in the pool yet." }
                            } else {
                                p { class: "text-gray-500 mb-2", "{filtered.len()} / {total} Emblems" }
                            }
                            for h in filtered.into_iter() {
                                HolderRow { key: "{h.gid}", holder: h }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! { p { class: "text-red-500", "Error: {err}" } },
                    None => rsx! { p { class: "text-gray-400", "Loading bond holders..." } },
                }
            }
        }
    }
}

#[component]
fn HolderRow(holder: BondHolderInfo) -> Element {
    let mut open = use_signal(|| false);
    let arrow = if open() { "▾" } else { "▸" };
    let bond_count = holder.bonds.len();

    rsx! {
        div { class: "border-b border-gray-700/50",
            button {
                class: "w-full flex items-center gap-2 py-1 text-left hover:bg-gray-900/40 rounded",
                onclick: move |_| open.toggle(),
                span { class: "text-gray-500 w-4 shrink-0", "{arrow}" }
                span { class: "text-white flex-1 truncate", title: "{holder.gid}", "{holder.name}" }
                span { class: "text-gray-500 w-20 text-right", "{bond_count} bonds" }
                span { class: "text-yellow-300 w-24 text-right", title: "Max level", "Lv {holder.max_level}" }
                span { class: "text-indigo-300 w-24 text-right", title: "Units at A rank", "{holder.a_rank_count} @ A" }
            }
            if open() {
                div { class: "pl-6 pb-1",
                    if holder.bonds.is_empty() {
                        p { class: "text-gray-500 py-0.5", "No bonds recorded." }
                    } else {
                        div { class: "flex items-center gap-2 py-0.5 text-gray-500",
                            span { class: "flex-1", "Pid" }
                            span { class: "w-10 text-center", "Rank" }
                            span { class: "w-20 text-right", "Level" }
                            span { class: "w-28 text-right", "Exp" }
                        }
                        for b in holder.bonds.iter() {
                            div { key: "{b.pid}", class: "flex items-center gap-2 py-0.5",
                                span { class: "text-gray-300 flex-1 truncate", "{b.pid}" }
                                span { class: "text-indigo-300 w-10 text-center", title: "Reliance / max", "{b.reliance}/{b.max_reliance}" }
                                span { class: "text-yellow-300 w-20 text-right", "{b.level}/{b.max_level}" }
                                span { class: "text-gray-400 w-28 text-right", title: "Exp toward next level", "{b.exp} ({b.current_level_exp}\u{2192}{b.next_level_exp})" }
                            }
                        }
                    }
                }
            }
        }
    }
}
