mod bond_inspector;
mod inventory_inspector;
mod skill_inspector;
mod stat_field;
mod stat_inspector;
mod unit_inspector;

use dioxus::prelude::*;

use crate::components::catalog_provider::Catalogs;
use crate::components::toast::use_toasts;
use crate::components::ui::{
    Card, EmptyState, ListRow, Page, SearchField, SectionLabel, StateKind,
};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    ClassInfo, ForceInfo, GetForcesRequest, GetUnitsRequest, ItemCatalogEntry, MoveUnitRequest,
    SetActedRequest, SetClassRequest, UnitSummary,
};
use crate::rpc;
pub use unit_inspector::UnitInspector;

// faction colors, matching the unit dots on the Map grid so a force reads the same in both places.
// player blue, enemy red, ally green, anything else gray
pub fn force_dot(id: i32) -> &'static str {
    match id {
        0 => "bg-blue-500",
        1 => "bg-red-500",
        2 => "bg-green-500",
        3 => "bg-purple-500",
        _ => "bg-gray-500",
    }
}

pub fn force_text(id: i32) -> &'static str {
    match id {
        0 => "text-blue-300",
        1 => "text-red-300",
        2 => "text-green-300",
        3 => "text-purple-300",
        _ => "text-gray-200",
    }
}

pub fn force_label(id: i32) -> &'static str {
    match id {
        0 => "Player",
        1 => "Enemy",
        2 => "Ally",
        _ => "",
    }
}

#[component]
pub fn ForceView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let catalogs = use_context::<Signal<Catalogs>>();
    let toasts = use_toasts();
    let mut forces = use_signal(|| None::<Result<Vec<ForceInfo>, String>>);
    let mut selected = use_signal(|| None::<i32>);
    let mut units = use_signal(|| None::<Result<Vec<UnitSummary>, String>>);
    let mut units_loading = use_signal(|| false);
    let mut mounted = use_signal(|| false);

    if !mounted() {
        mounted.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetForcesRequest).await.map(|r| r.forces);
            forces.set(Some(result));
        });
    }

    let select_force = use_callback(move |id: i32| {
        selected.set(Some(id));
        units.set(None);
        units_loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetUnitsRequest { force_id: id }).await.map(|r| r.units);
            units.set(Some(result));
            units_loading.set(false);
        });
    });

    let on_class_change = move |req: SetClassRequest| {
        spawn(async move {
            match rpc::call(&conn, req.clone()).await {
                Ok(resp) => units.with_mut(|slot| {
                    if let Some(Ok(list)) = slot.as_mut() {
                        if let Some(u) = list.iter_mut().find(|u| u.index == req.unit_index) {
                            u.class_jid = resp.class_jid;
                        }
                    }
                }),
                Err(e) => toasts.show(format!("Class change failed: {e}")),
            }
        });
    };

    let on_acted = move |req: SetActedRequest| {
        spawn(async move {
            match rpc::call(&conn, req.clone()).await {
                Ok(resp) => units.with_mut(|slot| {
                    if let Some(Ok(list)) = slot.as_mut() {
                        if let Some(u) = list.iter_mut().find(|u| u.index == req.unit_index) {
                            u.acted = resp.acted;
                        }
                    }
                }),
                Err(e) => toasts.show(format!("Could not change acted: {e}")),
            }
        });
    };

    let on_move = move |req: MoveUnitRequest| {
        spawn(async move {
            match rpc::call(&conn, req.clone()).await {
                Ok(_) => {
                    if let Ok(u) = rpc::call(&conn, GetUnitsRequest { force_id: req.from_force_id }).await {
                        units.set(Some(Ok(u.units)));
                    }
                    if let Ok(f) = rpc::call(&conn, GetForcesRequest).await {
                        forces.set(Some(Ok(f.forces)));
                    }
                }
                Err(e) => toasts.show(format!("Move failed: {e}")),
            }
        });
    };

    let force_options = forces().and_then(|r| r.ok()).unwrap_or_default();

    rsx! {
        Page { row: true,
            // left pane: force selector
            Card {
                title: "Forces",
                class: "w-44 shrink-0",
                padded: false,
                match forces() {
                    Some(Ok(list)) => rsx! {
                        for f in list.into_iter() {
                            ForceButton {
                                key: "{f.id}",
                                force: f,
                                selected: selected(),
                                on_select: move |id| select_force.call(id),
                            }
                        }
                    },
                    Some(Err(err)) => rsx! {
                        EmptyState { kind: StateKind::Error, message: "Error: {err}", compact: true }
                    },
                    None => rsx! {
                        EmptyState { kind: StateKind::Loading, message: "Loading forces\u{2026}", compact: true }
                    },
                }
            }
            // right pane: units list
            UnitsPanel {
                units: units(),
                force_id: selected(),
                loading: units_loading(),
                classes: catalogs().classes,
                item_catalog: catalogs().items,
                force_options: force_options,
                on_class_change: on_class_change,
                on_move: on_move,
                on_acted: on_acted,
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ForceButtonProps {
    pub force: ForceInfo,
    pub selected: Option<i32>,
    pub on_select: EventHandler<i32>,
}

#[component]
pub fn ForceButton(props: ForceButtonProps) -> Element {
    let id = props.force.id;
    let active = props.selected == Some(id);
    let on_select = props.on_select;
    rsx! {
        ListRow {
            selected: active,
            onclick: move |_| on_select.call(id),
            span { class: "h-2.5 w-2.5 rounded-full shrink-0 {force_dot(id)}" }
            span { class: "text-sm flex-1 truncate {force_text(id)}", "{props.force.label}" }
            span { class: "text-gray-500 text-xs shrink-0", "({props.force.unit_count})" }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct UnitsPanelProps {
    pub units: Option<Result<Vec<UnitSummary>, String>>,
    pub force_id: Option<i32>,
    pub loading: bool,
    pub classes: Vec<ClassInfo>,
    pub item_catalog: Vec<ItemCatalogEntry>,
    pub force_options: Vec<ForceInfo>,
    pub on_class_change: EventHandler<SetClassRequest>,
    pub on_move: EventHandler<MoveUnitRequest>,
    pub on_acted: EventHandler<SetActedRequest>,
}

#[component]
pub fn UnitsPanel(props: UnitsPanelProps) -> Element {
    let mut search = use_signal(String::new);
    rsx! {
        Card {
            class: "flex-1",
            padded: false,
            header: rsx! {
                SearchField {
                    value: search(),
                    placeholder: "Filter units\u{2026}",
                    class: "w-56",
                    on_input: move |v| search.set(v),
                }
            },
            if props.force_id.is_none() {
                EmptyState { kind: StateKind::Empty, message: "Pick a force to see its units." }
            } else if props.loading {
                EmptyState { kind: StateKind::Loading, message: "Loading units\u{2026}" }
            } else {
                match props.units.as_ref() {
                    Some(Ok(list)) if list.is_empty() => rsx! {
                        EmptyState { kind: StateKind::Empty, message: "No units in this force." }
                    },
                    Some(Ok(list)) => {
                        let query = search().to_lowercase();
                        let filtered: Vec<_> = list.iter()
                            .filter(|u| query.is_empty()
                                || u.name.to_lowercase().contains(&query)
                                || u.class_jid.to_lowercase().contains(&query))
                            .cloned()
                            .collect();
                        let shown = filtered.len();
                        rsx! {
                            div { class: "px-3 pb-1 pt-2 shrink-0",
                                SectionLabel { label: "{shown} units" }
                            }
                            div { class: "flex-1 min-h-0 overflow-auto",
                                for u in filtered.into_iter() {
                                    UnitInspector {
                                        key: "{u.index}",
                                        force_id: props.force_id.unwrap_or(0),
                                        unit: u,
                                        classes: props.classes.clone(),
                                        item_catalog: props.item_catalog.clone(),
                                        force_options: props.force_options.clone(),
                                        on_class_change: props.on_class_change,
                                        on_move: props.on_move,
                                        on_acted: props.on_acted,
                                    }
                                }
                                if shown == 0 {
                                    EmptyState { kind: StateKind::Empty, message: "No units match the filter." }
                                }
                            }
                        }
                    },
                    Some(Err(err)) => rsx! {
                        EmptyState { kind: StateKind::Error, message: "Error: {err}" }
                    },
                    None => rsx! {},
                }
            }
        }
    }
}
