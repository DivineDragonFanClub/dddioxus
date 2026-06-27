use dioxus::prelude::*;

use super::{IconPicker, PickerOption, SpriteImg};
use crate::components::ui::{Button, ButtonSize, ButtonVariant, EditableNumber, EmptyState, ListRow, Select, SelectSize, StateKind, Tone};
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

    let catalog = props.item_catalog.clone();

    rsx! {
        div { class: "mt-2",
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                onclick: toggle,
                if open() { "\u{25BE} Items" } else { "\u{25B8} Items" }
            }
            if open() {
                div { class: "mt-1 pl-2 border-l border-gray-700",
                    match items() {
                        None => rsx! {
                            EmptyState { kind: StateKind::Loading, message: "Loading items\u{2026}", compact: true }
                        },
                        Some(list) if list.is_empty() => rsx! {
                            EmptyState { kind: StateKind::Empty, message: "No items.", compact: true }
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
                    {
                        // add_kind starts at 0 (not a real kind), so fall back to the first
                        // available kind, which is what the native select shows by default.
                        // keeps the visible kind and the picker's filter in sync.
                        let kinds = kinds_in_catalog(&catalog);
                        let effective_kind = if kinds.contains(&add_kind()) {
                            add_kind()
                        } else {
                            kinds.first().copied().unwrap_or(0)
                        };
                        rsx! {
                            div { class: "flex items-center gap-1 mt-1",
                                Select {
                                    size: SelectSize::Sm,
                                    on_change: move |v: String| {
                                        if let Ok(k) = v.parse::<i32>() { add_kind.set(k); }
                                    },
                                    for k in kinds.iter().copied() {
                                        option { value: "{k}", selected: k == effective_kind, "{item_kind_label(k)}" }
                                    }
                                }
                                IconPicker {
                                    placeholder: "+ Add item\u{2026}".to_string(),
                                    options: catalog.iter().filter(|c| c.kind == effective_kind).map(|c| PickerOption {
                                        value: c.iid.clone(),
                                        label: c.name.clone(),
                                        icon: if c.icon.is_empty() { None } else { Some(format!("/sprite/item/{}.png", c.icon)) },
                                    }).collect::<Vec<_>>(),
                                    on_select: move |v: String| add(v),
                                }
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
        ListRow {
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                title: "Equip",
                onclick: move |_| on_equip.call(idx),
                span {
                    class: if equipped { "text-amber-300" } else { "text-gray-600" },
                    "\u{2605}"
                }
            }
            if !props.item.icon.is_empty() {
                SpriteImg { src: "/sprite/item/{props.item.icon}.png", class: "w-6 h-6 object-contain shrink-0" }
            }
            span { class: "text-white flex-1 truncate text-xs", "{props.item.name}" }
            EditableNumber {
                value: props.item.endurance as i64,
                width: "w-12",
                on_commit: move |v: i64| on_uses.call((idx, v as i32)),
            }
            Button {
                tone: Tone::Red,
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                title: "Remove",
                onclick: move |_| on_remove.call(idx),
                "\u{2715}"
            }
        }
    }
}
