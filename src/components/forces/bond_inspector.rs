use dioxus::prelude::*;

use crate::components::ui::{Button, ButtonSize, ButtonVariant, EditableNumber, EmptyState, ListRow, StateKind};
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
                            b.reliance = resp.reliance;
                        }
                    }
                });
            }
        });
    };

    rsx! {
        div { class: "mt-1",
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                onclick: toggle,
                if open() { "\u{25BE} Bonds" } else { "\u{25B8} Bonds" }
            }
            if open() {
                div { class: "mt-1 pl-2 border-l border-gray-700",
                    match bonds() {
                        None => rsx! {
                            EmptyState { kind: StateKind::Loading, message: "Loading bonds\u{2026}", compact: true }
                        },
                        Some(list) if list.is_empty() => rsx! {
                            EmptyState { kind: StateKind::Empty, message: "No Emblem bonds.", compact: true }
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
    let gid = props.bond.gid.clone();
    let current_level = props.bond.level;
    let on_commit = props.on_commit;

    rsx! {
        ListRow {
            span { class: "text-white flex-1 truncate text-xs", title: "{props.bond.gid}", "{props.bond.name}" }
            span { class: "text-indigo-300 w-6 shrink-0 text-center text-xs", title: "Reliance rank", "{props.bond.reliance}" }
            div { class: "flex items-center gap-1 shrink-0",
                span { class: "text-gray-500 text-xs", "Lv" }
                EditableNumber {
                    value: current_level as i64,
                    width: "w-9",
                    on_commit: move |v: i64| on_commit.call((gid.clone(), v as i32)),
                }
                span { class: "text-gray-600 text-xs", "/{props.bond.max_level}" }
            }
            span { class: "text-gray-500 text-xs w-20 shrink-0 text-right", title: "Bond exp", "{props.bond.exp} exp" }
        }
    }
}
