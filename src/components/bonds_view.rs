use dioxus::prelude::*;

use crate::components::toast::use_toasts;
use crate::components::ui::{
    Button, Card, EditableNumber, EmptyState, ListRow, Page, SearchField, SectionLabel, StateKind,
};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    BondHolderInfo, GetBondHoldersRequest, GetBondHoldersResponse, HolderBond, SetHolderBondRequest,
    SetHolderBondResponse,
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
        Page {
            Card {
                class: "flex-1",
                padded: false,
                header: rsx! {
                    SearchField {
                        value: search(),
                        placeholder: "Filter holders\u{2026}",
                        class: "w-56",
                        on_input: move |v| search.set(v),
                    }
                    Button {
                        disabled: loading(),
                        onclick: move |_| fetch(),
                        if loading() { "Refreshing\u{2026}" } else { "Refresh" }
                    }
                },
                div { class: "p-3",
                    match data() {
                        None => rsx! { EmptyState { kind: StateKind::Loading, message: "Loading bond holders\u{2026}" } },
                        Some(Err(err)) => rsx! {
                            EmptyState { kind: StateKind::Error, message: "Error: {err}" }
                        },
                        Some(Ok(resp)) => {
                            let query = search().to_lowercase();
                            let total = resp.holders.len();
                            let filtered: Vec<_> = resp.holders.into_iter()
                                .filter(|h| query.is_empty()
                                    || h.name.to_lowercase().contains(&query)
                                    || h.gid.to_lowercase().contains(&query))
                                .collect();
                            if total == 0 {
                                rsx! { EmptyState { kind: StateKind::Empty, message: "No bond holders in the pool yet." } }
                            } else {
                                rsx! {
                                    SectionLabel { label: "{filtered.len()} / {total} Emblems", class: "mb-2" }
                                    for h in filtered.into_iter() {
                                        HolderRow { key: "{h.gid}", holder: h }
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

#[component]
fn HolderRow(holder: BondHolderInfo) -> Element {
    let mut open = use_signal(|| false);
    let arrow = if open() { "\u{25BE}" } else { "\u{25B8}" };
    let bond_count = holder.bonds.len();

    rsx! {
        div {
            ListRow {
                onclick: move |_| open.toggle(),
                span { class: "text-gray-500 w-4 shrink-0 select-none", "{arrow}" }
                span { class: "text-white flex-1 truncate", title: "{holder.gid}", "{holder.name}" }
                span { class: "text-gray-500 text-xs w-20 text-right shrink-0", "{bond_count} bonds" }
                span { class: "text-amber-300 text-xs w-24 text-right shrink-0", title: "Max level", "Lv {holder.max_level}" }
                span { class: "text-indigo-300 text-xs w-24 text-right shrink-0", title: "Units at A rank", "{holder.a_rank_count} @ A" }
            }
            if open() {
                div { class: "pl-8 pb-1",
                    if holder.bonds.is_empty() {
                        EmptyState { kind: StateKind::Empty, message: "No bonds recorded.", compact: true }
                    } else {
                        div { class: "flex items-center gap-2 py-0.5 text-gray-500 text-xs",
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
        b.reliance = r.reliance;
        // max_level / max_reliance are static per bond, leave them as the getter set them
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
    let toasts = use_toasts();
    // live values, updated from each edit's read-back (level and exp drag each other server-side)
    let cur = use_signal(|| props.bond.clone());
    let gid = props.gid.clone();
    let pid = props.bond.pid.clone();

    let commit_level = {
        let (gid, pid) = (gid.clone(), pid.clone());
        move |v: i64| {
            let req = SetHolderBondRequest { gid: gid.clone(), pid: pid.clone(), level: Some(v as i32), exp: None };
            spawn(async move {
                match rpc::call(&conn, req).await {
                    Ok(r) => apply_bond(cur, r),
                    Err(e) => toasts.show(format!("Bond level edit failed: {e}")),
                }
            });
        }
    };
    let commit_exp = {
        let (gid, pid) = (gid.clone(), pid.clone());
        move |v: i64| {
            let req = SetHolderBondRequest { gid: gid.clone(), pid: pid.clone(), level: None, exp: Some(v as i32) };
            spawn(async move {
                match rpc::call(&conn, req).await {
                    Ok(r) => apply_bond(cur, r),
                    Err(e) => toasts.show(format!("Bond exp edit failed: {e}")),
                }
            });
        }
    };

    let b = cur();
    rsx! {
        ListRow {
            span { class: "text-gray-300 flex-1 truncate text-xs", title: "{b.pid}", "{b.name}" }
            span { class: "text-indigo-300 text-xs w-10 text-center", title: "Reliance / max", "{b.reliance}/{b.max_reliance}" }
            div { class: "w-24 flex items-center justify-end gap-1",
                EditableNumber { value: b.level as i64, width: "w-12", on_commit: commit_level }
                span { class: "text-gray-500 text-[10px]", "/{b.max_level}" }
            }
            div { class: "w-32 flex items-center justify-end gap-1",
                EditableNumber { value: b.exp as i64, width: "w-16", on_commit: commit_exp }
                span { class: "text-gray-500 text-[10px]", "({b.current_level_exp}\u{2192}{b.next_level_exp})" }
            }
        }
    }
}
