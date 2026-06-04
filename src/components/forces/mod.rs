mod bond_inspector;
mod inventory_inspector;
mod skill_inspector;
mod stat_field;
mod stat_inspector;
mod unit_inspector;

use dioxus::prelude::*;

use crate::components::catalog_provider::Catalogs;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    ClassInfo, ForceInfo, GetForcesRequest, GetUnitsRequest, ItemCatalogEntry, MoveUnitRequest,
    SetActedRequest, SetClassRequest, UnitSummary,
};
use crate::rpc;
pub use unit_inspector::UnitInspector;

#[component]
pub fn ForceView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let catalogs = use_context::<Signal<Catalogs>>();
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
            if let Ok(resp) = rpc::call(&conn, req.clone()).await {
                units.with_mut(|slot| {
                    if let Some(Ok(list)) = slot.as_mut() {
                        if let Some(u) = list.iter_mut().find(|u| u.index == req.unit_index) {
                            u.class_jid = resp.class_jid;
                        }
                    }
                });
            }
        });
    };

    let on_acted = move |req: SetActedRequest| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, req.clone()).await {
                units.with_mut(|slot| {
                    if let Some(Ok(list)) = slot.as_mut() {
                        if let Some(u) = list.iter_mut().find(|u| u.index == req.unit_index) {
                            u.acted = resp.acted;
                        }
                    }
                });
            }
        });
    };

    let on_move = move |req: MoveUnitRequest| {
        spawn(async move {
            if rpc::call(&conn, req.clone()).await.is_ok() {
                if let Ok(u) = rpc::call(&conn, GetUnitsRequest { force_id: req.from_force_id }).await {
                    units.set(Some(Ok(u.units)));
                }
                if let Ok(f) = rpc::call(&conn, GetForcesRequest).await {
                    forces.set(Some(Ok(f.forces)));
                }
            }
        });
    };

    let force_options = forces().and_then(|r| r.ok()).unwrap_or_default();

    rsx! {
        div { class: "flex flex-1 min-h-0",
            div { class: "w-44 shrink-0 bg-gray-900 border-r border-gray-700 overflow-y-auto",
                div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                    h2 { class: "text-white font-bold text-sm", "Forces" }
                }
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
                        p { class: "text-red-500 text-xs p-3", "Error: {err}" }
                    },
                    None => rsx! {
                        p { class: "text-gray-400 text-xs p-3", "Loading forces..." }
                    },
                }
            }
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
    let class = if active {
        "w-full text-left px-3 py-2 text-sm bg-indigo-600 text-white"
    } else {
        "w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-gray-800"
    };
    let on_select = props.on_select;
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| on_select.call(id),
            "{props.force.label} "
            span { class: "text-gray-400", "({props.force.unit_count})" }
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
    rsx! {
        div { class: "flex-1 min-h-0 overflow-auto bg-gray-800",
            if props.force_id.is_none() {
                div { class: "p-6 text-gray-500 text-sm", "Pick a force to see its units." }
            } else if props.loading {
                div { class: "p-6 text-gray-400 text-sm", "Loading units..." }
            } else {
                match props.units.as_ref() {
                    Some(Ok(list)) if list.is_empty() => rsx! {
                        div { class: "p-6 text-gray-500 text-sm", "No units in this force." }
                    },
                    Some(Ok(list)) => rsx! {
                        for u in list.clone().into_iter() {
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
                    },
                    Some(Err(err)) => rsx! {
                        p { class: "text-red-500 text-sm p-6", "Error: {err}" }
                    },
                    None => rsx! {},
                }
            }
        }
    }
}
