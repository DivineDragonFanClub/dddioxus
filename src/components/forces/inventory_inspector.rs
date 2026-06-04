use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    item_kind_label, AddItemRequest, EquipItemRequest, GetUnitItemsRequest, ItemCatalogEntry, RemoveItemRequest,
    SetEnduranceRequest, UnitItemInfo,
};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct InventoryInspectorProps {
    pub force_id: i32,
    pub unit_index: i32,
    pub item_catalog: Vec<ItemCatalogEntry>,
}

#[component]
pub fn InventoryInspector(props: InventoryInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut open = use_signal(|| false);
    let mut items = use_signal(|| None::<Vec<UnitItemInfo>>);
    let mut loaded = use_signal(|| false);
    let mut add_kind = use_signal(|| 0i32);

    let force_id = props.force_id;
    let unit_index = props.unit_index;

    let load = use_callback(move |_: ()| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetUnitItemsRequest { force_id, unit_index }).await {
                items.set(Some(resp.items));
            }
        });
    });

    let toggle = move |_| {
        let now_open = !open();
        open.set(now_open);
        if now_open && !loaded() {
            loaded.set(true);
            load.call(());
        }
    };

    let add = move |iid: String| {
        spawn(async move {
            if rpc::call(&conn, AddItemRequest { force_id, unit_index, iid }).await.is_ok() {
                load.call(());
            }
        });
    };
    let remove = move |item_index: i32| {
        spawn(async move {
            if rpc::call(&conn, RemoveItemRequest { force_id, unit_index, item_index }).await.is_ok() {
                load.call(());
            }
        });
    };
    let equip = move |item_index: i32| {
        spawn(async move {
            if rpc::call(&conn, EquipItemRequest { force_id, unit_index, item_index }).await.is_ok() {
                load.call(());
            }
        });
    };
    let set_uses = move |(item_index, value): (i32, i32)| {
        spawn(async move {
            if rpc::call(&conn, SetEnduranceRequest { force_id, unit_index, item_index, value }).await.is_ok() {
                load.call(());
            }
        });
    };

    let arrow = if open() { "▾" } else { "▸" };
    let catalog = props.item_catalog.clone();

    rsx! {
        div { class: "mt-2",
            button {
                class: "text-gray-400 hover:text-gray-200 text-xs",
                onclick: toggle,
                "{arrow} Items"
            }
            if open() {
                div { class: "mt-1 pl-2 border-l border-gray-700",
                    match items() {
                        None => rsx! { p { class: "text-gray-500 text-xs py-1", "Loading items..." } },
                        Some(list) if list.is_empty() => rsx! {
                            p { class: "text-gray-500 text-xs py-1", "No items." }
                        },
                        Some(list) => rsx! {
                            for it in list.into_iter() {
                                ItemRow {
                                    key: "{it.index}-{it.iid}",
                                    item: it,
                                    on_equip: move |idx| equip(idx),
                                    on_remove: move |idx| remove(idx),
                                    on_uses: move |pair| set_uses(pair),
                                }
                            }
                        },
                    }
                    div { class: "flex items-center gap-1 mt-1",
                        select {
                            class: "bg-gray-900 text-gray-300 text-xs rounded border border-gray-600 px-1 py-0.5",
                            onchange: move |e| {
                                if let Ok(k) = e.value().parse::<i32>() { add_kind.set(k); }
                            },
                            for k in kinds_in_catalog(&catalog).into_iter() {
                                option { value: "{k}", selected: k == add_kind(), "{item_kind_label(k)}" }
                            }
                        }
                        select {
                            class: "bg-gray-900 text-emerald-300 text-xs rounded border border-gray-600 px-1 py-0.5 flex-1",
                            onchange: move |e| {
                                let iid = e.value();
                                if !iid.is_empty() { add(iid); }
                            },
                            option { value: "", selected: true, disabled: true, "+ Add item…" }
                            for c in catalog.iter().filter(|c| c.kind == add_kind()) {
                                option { value: "{c.iid}", "{c.name}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn kinds_in_catalog(catalog: &[ItemCatalogEntry]) -> Vec<i32> {
    let mut kinds: Vec<i32> = catalog.iter().map(|c| c.kind).collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}

#[derive(PartialEq, Clone, Props)]
pub struct ItemRowProps {
    pub item: UnitItemInfo,
    pub on_equip: EventHandler<i32>,
    pub on_remove: EventHandler<i32>,
    pub on_uses: EventHandler<(i32, i32)>,
}

#[component]
pub fn ItemRow(props: ItemRowProps) -> Element {
    let idx = props.item.index;
    let equipped = props.item.equipped;
    let on_equip = props.on_equip;
    let on_remove = props.on_remove;
    let on_uses = props.on_uses;

    rsx! {
        div { class: "flex items-center gap-2 py-0.5 text-xs",
            button {
                class: if equipped { "text-yellow-400" } else { "text-gray-600 hover:text-gray-400" },
                title: "Equip",
                onclick: move |_| on_equip.call(idx),
                "★"
            }
            span { class: "text-gray-500 w-14 shrink-0", "{item_kind_label(props.item.kind)}" }
            span { class: "text-white flex-1 truncate", "{props.item.name}" }
            UsesField {
                item_index: idx,
                value: props.item.endurance,
                on_uses,
            }
            button {
                class: "text-red-500 hover:text-red-300",
                title: "Remove",
                onclick: move |_| on_remove.call(idx),
                "✕"
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct UsesFieldProps {
    pub item_index: i32,
    pub value: i32,
    pub on_uses: EventHandler<(i32, i32)>,
}

#[component]
pub fn UsesField(props: UsesFieldProps) -> Element {
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.value.to_string());

    let commit = {
        let on_uses = props.on_uses;
        let item_index = props.item_index;
        let current = props.value;
        move || {
            editing.set(false);
            if let Ok(v) = draft().trim().parse::<i32>() {
                if v != current {
                    on_uses.call((item_index, v));
                }
            }
        }
    };

    rsx! {
        if editing() {
            input {
                r#type: "number",
                class: "w-12 px-1 bg-gray-900 text-yellow-300 rounded border border-gray-600 text-center",
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
                class: "w-12 text-center text-yellow-300 cursor-text hover:bg-gray-900 rounded",
                title: "Uses",
                onclick: {
                    let value = props.value;
                    move |_| {
                        draft.set(value.to_string());
                        editing.set(true);
                    }
                },
                "{props.value}"
            }
        }
    }
}
