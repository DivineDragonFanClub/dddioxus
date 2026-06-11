use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    BondHolderInfo, GetBondHoldersRequest, GetBondHoldersResponse, HolderBond, SetHolderBondRequest, SetHolderBondResponse,
};
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
                            span { class: "flex-1", "Unit" }
                            span { class: "w-10 text-center", "Rank" }
                            span { class: "w-24 text-right", "Level" }
                            span { class: "w-32 text-right", "Exp" }
                        }
                        for b in holder.bonds.iter() {
                            BondRow { key: "{b.pid}", gid: holder.gid.clone(), bond: b.clone() }
                        }
                    }
                }
            }
        }
    }
}

fn apply_bond(mut cur: Signal<HolderBond>, r: SetHolderBondResponse) {
    cur.with_mut(|b| {
        b.level = r.level;
        b.exp = r.exp;
        b.current_level_exp = r.current_level_exp;
        b.next_level_exp = r.next_level_exp;
        b.max_level = r.max_level;
        b.reliance = r.reliance;
        b.max_reliance = r.max_reliance;
    });
}

#[derive(PartialEq, Clone, Props)]
struct BondRowProps {
    gid: String,
    bond: HolderBond,
}

#[component]
fn BondRow(props: BondRowProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    // live values, updated from each edit's read-back (level and exp drag each other server-side)
    let cur = use_signal(|| props.bond.clone());
    let gid = props.gid.clone();
    let pid = props.bond.pid.clone();

    let commit_level = {
        let (gid, pid) = (gid.clone(), pid.clone());
        move |v: i32| {
            let req = SetHolderBondRequest { gid: gid.clone(), pid: pid.clone(), level: Some(v), exp: None };
            spawn(async move {
                if let Ok(r) = rpc::call(&conn, req).await {
                    apply_bond(cur, r);
                }
            });
        }
    };
    let commit_exp = {
        let (gid, pid) = (gid.clone(), pid.clone());
        move |v: i32| {
            let req = SetHolderBondRequest { gid: gid.clone(), pid: pid.clone(), level: None, exp: Some(v) };
            spawn(async move {
                if let Ok(r) = rpc::call(&conn, req).await {
                    apply_bond(cur, r);
                }
            });
        }
    };

    let b = cur();
    rsx! {
        div { class: "flex items-center gap-2 py-0.5",
            span { class: "text-gray-300 flex-1 truncate", title: "{b.pid}", "{b.name}" }
            span { class: "text-indigo-300 w-10 text-center", title: "Reliance / max", "{b.reliance}/{b.max_reliance}" }
            div { class: "w-24 flex items-center justify-end gap-1",
                NumEdit { value: b.level, on_commit: commit_level }
                span { class: "text-gray-500 text-[10px]", "/{b.max_level}" }
            }
            div { class: "w-32 flex items-center justify-end gap-1",
                NumEdit { value: b.exp, on_commit: commit_exp }
                span { class: "text-gray-500 text-[10px]", "({b.current_level_exp}\u{2192}{b.next_level_exp})" }
            }
        }
    }
}

// small inline integer field, commits on Enter or blur
#[derive(PartialEq, Clone, Props)]
struct NumEditProps {
    value: i32,
    on_commit: EventHandler<i32>,
}

#[component]
fn NumEdit(props: NumEditProps) -> Element {
    let mut draft = use_signal(|| props.value.to_string());
    let mut last = use_signal(|| props.value);
    if last() != props.value {
        last.set(props.value);
        draft.set(props.value.to_string());
    }

    let commit = {
        let on_commit = props.on_commit;
        let current = props.value;
        move || {
            if let Ok(v) = draft().trim().parse::<i32>() {
                if v != current {
                    on_commit.call(v);
                }
            }
        }
    };

    rsx! {
        input {
            r#type: "number",
            class: "w-16 px-1 py-0.5 bg-gray-900 text-yellow-300 rounded border border-gray-700 focus:border-indigo-500 focus:outline-none text-right text-xs",
            value: "{draft}",
            oninput: move |e| draft.set(e.value()),
            onblur: {
                let commit = commit.clone();
                move |_| commit()
            },
            onkeydown: {
                let commit = commit.clone();
                move |e| {
                    if e.key() == Key::Enter {
                        commit();
                    }
                }
            },
        }
    }
}
