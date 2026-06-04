use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{BondInfo, GetUnitBondsRequest, SetBondLevelRequest};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct BondInspectorProps {
    pub force_id: i32,
    pub unit_index: i32,
}

#[component]
pub fn BondInspector(props: BondInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut open = use_signal(|| false);
    let mut bonds = use_signal(|| None::<Vec<BondInfo>>);
    let mut loaded = use_signal(|| false);

    let force_id = props.force_id;
    let unit_index = props.unit_index;

    let toggle = move |_| {
        let now_open = !open();
        open.set(now_open);
        if now_open && !loaded() {
            loaded.set(true);
            spawn(async move {
                if let Ok(resp) = rpc::call(&conn, GetUnitBondsRequest { force_id, unit_index }).await {
                    bonds.set(Some(resp.bonds));
                }
            });
        }
    };

    let on_commit = move |(gid, level): (String, i32)| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, SetBondLevelRequest { force_id, unit_index, gid: gid.clone(), level }).await {
                bonds.with_mut(|slot| {
                    if let Some(list) = slot.as_mut() {
                        if let Some(b) = list.iter_mut().find(|b| b.gid == gid) {
                            b.level = resp.level;
                            b.exp = resp.exp;
                            b.max_level = resp.max_level;
                            b.reliance = resp.reliance;
                        }
                    }
                });
            }
        });
    };

    let arrow = if open() { "▾" } else { "▸" };

    rsx! {
        div { class: "mt-1",
            button {
                class: "text-gray-400 hover:text-gray-200 text-xs",
                onclick: toggle,
                "{arrow} Bonds"
            }
            if open() {
                div { class: "mt-1 pl-2 border-l border-gray-700",
                    match bonds() {
                        None => rsx! { p { class: "text-gray-500 text-xs py-1", "Loading bonds..." } },
                        Some(list) if list.is_empty() => rsx! {
                            p { class: "text-gray-500 text-xs py-1", "No Emblem bonds." }
                        },
                        Some(list) => rsx! {
                            for b in list.into_iter() {
                                BondRow { key: "{b.gid}", bond: b, on_commit: on_commit }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct BondRowProps {
    pub bond: BondInfo,
    pub on_commit: EventHandler<(String, i32)>,
}

#[component]
pub fn BondRow(props: BondRowProps) -> Element {
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.bond.level.to_string());

    let commit = {
        let on_commit = props.on_commit;
        let gid = props.bond.gid.clone();
        let current = props.bond.level;
        move || {
            editing.set(false);
            if let Ok(v) = draft().trim().parse::<i32>() {
                if v != current {
                    on_commit.call((gid.clone(), v));
                }
            }
        }
    };

    rsx! {
        div { class: "flex items-center gap-2 py-0.5 text-xs",
            span { class: "text-white flex-1 truncate", title: "{props.bond.gid}", "{props.bond.name}" }
            span { class: "text-indigo-300 w-6 shrink-0 text-center", title: "Reliance rank", "{props.bond.reliance}" }
            div { class: "w-16 shrink-0 flex items-center gap-1",
                span { class: "text-gray-500", "Lv" }
                if editing() {
                    input {
                        r#type: "number",
                        class: "w-9 px-1 bg-gray-900 text-yellow-300 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center",
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
                        class: "text-yellow-300 cursor-text hover:bg-gray-900 rounded px-0.5",
                        onclick: {
                            let value = props.bond.level;
                            move |_| {
                                draft.set(value.to_string());
                                editing.set(true);
                            }
                        },
                        "{props.bond.level}"
                    }
                }
                span { class: "text-gray-600", "/{props.bond.max_level}" }
            }
            span { class: "text-gray-500 w-20 shrink-0 text-right", title: "Bond exp", "{props.bond.exp} exp" }
        }
    }
}
