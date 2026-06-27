mod bond_inspector;
mod inventory_inspector;
mod skill_inspector;
mod stat_field;
mod stat_inspector;
mod unit_inspector;

use std::rc::Rc;

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
#[derive(Clone, PartialEq)]
pub struct PickerOption {
    pub value: String,
    pub label: String,
    pub icon: Option<String>, // full sprite src, or None for no icon
}

// where the popup floats relative to the trigger. `edge` is a top offset when !up,
// or a bottom offset (distance from viewport bottom) when up
#[derive(Clone, Copy, PartialEq, Default)]
struct PopupPos {
    left: f64,
    width: f64,
    edge: f64,
    max_h: f64,
    up: bool,
}

// a dropdown that shows an icon next to each option (native <select> can't). the list
// is position:fixed anchored to the button, flips up when there's more room above, and
// caps its height to the space available, so a small window can't trap it off-screen
#[component]
pub fn IconPicker(placeholder: String, options: Vec<PickerOption>, on_select: EventHandler<String>) -> Element {
    let mut open = use_signal(|| false);
    let mut pos = use_signal(PopupPos::default);
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut query = use_signal(String::new);

    rsx! {
        div { class: "flex-1 min-w-0",
            button {
                class: "w-full flex items-center justify-between gap-1 px-2 py-1 text-xs rounded bg-indigo-600 hover:bg-indigo-500 text-white",
                onmounted: move |e: Event<MountedData>| trigger.set(Some(e.data())),
                onclick: move |_| {
                    if open() {
                        open.set(false);
                        return;
                    }
                    let Some(elem) = trigger() else { return };
                    spawn(async move {
                        let Ok(rect) = elem.get_client_rect().await else { return };
                        let mut eval = document::eval("dioxus.send([window.innerWidth, window.innerHeight])");
                        let (vw, vh) = eval.recv::<(f64, f64)>().await.unwrap_or((f64::MAX, f64::MAX));
                        const GAP: f64 = 4.0;
                        let btn_top = rect.origin.y;
                        let btn_bottom = rect.origin.y + rect.size.height;
                        let below = vh - btn_bottom - GAP;
                        let above = btn_top - GAP;
                        // open upward only when below is cramped and above has more room
                        let up = below < 180.0 && above > below;
                        let (edge, max_h) = if up {
                            (vh - btn_top + GAP, above.clamp(80.0, 320.0))
                        } else {
                            (btn_bottom + GAP, below.clamp(80.0, 320.0))
                        };
                        let width = rect.size.width.max(180.0);
                        let left = (rect.origin.x).min((vw - width - 4.0).max(4.0));
                        pos.set(PopupPos { left, width, edge, max_h, up });
                        query.set(String::new()); // start each open with an empty filter
                        open.set(true);
                    });
                },
                span { class: "truncate", "{placeholder}" }
                span { class: "text-indigo-200 shrink-0", "\u{25be}" }
            }
            if open() {
                div { class: "fixed inset-0 z-40", onclick: move |_| open.set(false) }
                {
                    let p = pos();
                    let edge = if p.up { format!("bottom:{}px", p.edge) } else { format!("top:{}px", p.edge) };
                    let q = query().to_lowercase();
                    let filtered: Vec<PickerOption> =
                        options.iter().filter(|o| q.is_empty() || o.label.to_lowercase().contains(&q)).cloned().collect();
                    // Enter on the filter box picks the first match
                    let first_val = filtered.first().map(|o| o.value.clone());
                    rsx! {
                        div {
                            class: "fixed z-50 flex flex-col overflow-hidden rounded border border-gray-700 bg-gray-800 shadow-xl shadow-black/40",
                            style: "left:{p.left}px; width:{p.width}px; max-height:{p.max_h}px; {edge}",
                            div { class: "p-1 border-b border-gray-700 shrink-0",
                                input {
                                    class: "w-full px-2 py-1 text-xs rounded bg-gray-900 text-white placeholder-gray-500 outline-none",
                                    placeholder: "Filter\u{2026}",
                                    value: "{query}",
                                    oninput: move |e| query.set(e.value()),
                                    onmounted: move |e: Event<MountedData>| {
                                        spawn(async move {
                                            let _ = e.data().set_focus(true).await;
                                        });
                                    },
                                    onkeydown: move |e: Event<KeyboardData>| match e.key() {
                                        Key::Enter => {
                                            if let Some(v) = first_val.clone() {
                                                on_select.call(v);
                                                open.set(false);
                                            }
                                        }
                                        Key::Escape => open.set(false),
                                        _ => {}
                                    },
                                }
                            }
                            div { class: "flex-1 min-h-0 overflow-auto",
                                if filtered.is_empty() {
                                    div { class: "px-2 py-1.5 text-gray-500 text-xs", "No matches" }
                                }
                                for opt in filtered.iter() {
                                    button {
                                        key: "{opt.value}",
                                        class: "flex items-center gap-2 w-full px-2 py-1 text-left hover:bg-gray-700",
                                        onclick: {
                                            let v = opt.value.clone();
                                            move |_| {
                                                on_select.call(v.clone());
                                                open.set(false);
                                            }
                                        },
                                        if let Some(src) = opt.icon.clone() {
                                            SpriteImg { src, class: "w-5 h-5 object-contain shrink-0" }
                                        } else {
                                            span { class: "w-5 h-5 shrink-0" }
                                        }
                                        span { class: "truncate text-white text-xs", "{opt.label}" }
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

// a sprite image that quietly disappears if its file 404s, instead of leaving a
// broken-image glyph in the row. used for item/skill icons that may not all exist
#[component]
pub fn SpriteImg(src: String, class: String) -> Element {
    let mut failed = use_signal(|| false);
    if failed() {
        return rsx! {};
    }
    rsx! {
        img { class: "{class}", src: "{src}", onerror: move |_| failed.set(true) }
    }
}

// where to load a unit's chibi from: a mod-provided png wins (sent inline as base64),
// then the baked base-game sprite, otherwise None and the caller shows a fallback
pub fn icon_src(icon: &str, icon_png: &Option<String>) -> Option<String> {
    if let Some(png) = icon_png {
        Some(format!("data:image/png;base64,{png}"))
    } else if !icon.is_empty() {
        Some(format!("/sprite/unit/{icon}.png"))
    } else {
        None
    }
}

pub fn force_dot(id: i32) -> &'static str {
    match id {
        0 => "bg-blue-500",
        1 => "bg-red-500",
        2 => "bg-green-500",
        3 => "bg-purple-500",
        _ => "bg-gray-500",
    }
}

// ring color for the chibi on the map, so affiliation stays readable
pub fn force_ring(id: i32) -> &'static str {
    match id {
        0 => "ring-blue-400",
        1 => "ring-red-400",
        2 => "ring-green-400",
        3 => "ring-purple-400",
        _ => "ring-gray-400",
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
