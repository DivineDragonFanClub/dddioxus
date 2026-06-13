use dioxus::prelude::*;

use crate::components::catalog_provider::Catalogs;
use crate::components::forces::UnitInspector;
use crate::components::globals_view::GlobalsView;
use crate::components::resizable_panel::ResizablePanel;
use crate::components::toast::use_toasts;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    CompleteMapRequest, ForceInfo, GetForcesRequest, MapGridRequest, MapPlacementsRequest, MapStatusRequest,
    MapTurnRequest, MapUnit, MoveUnitRequest, RewindCancelRequest, RewindCommitRequest, RewindEntriesRequest,
    RewindEntry, RewindPreviewRequest, SetActedRequest, SetClassRequest, SetMapTurnRequest, SetUnitPosRequest,
    UnitSummary,
};
use crate::rpc;

fn force_color(force_id: i32) -> &'static str {
    match force_id {
        0 => "bg-blue-500",
        1 => "bg-red-500",
        2 => "bg-green-500",
        _ => "bg-gray-500",
    }
}

fn force_label(force_id: i32) -> &'static str {
    match force_id {
        0 => "Player",
        1 => "Enemy",
        2 => "Ally",
        _ => "",
    }
}

#[component]
pub fn MapView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let catalogs = use_context::<Signal<Catalogs>>();
    let toasts = use_toasts();
    let mut status = use_signal(|| None::<bool>);
    let mut grid = use_signal(|| (0i32, 0i32));
    let mut placements = use_signal(Vec::<MapUnit>::new);
    let mut rewind = use_signal(Vec::<RewindEntry>::new);
    let mut previewing = use_signal(|| None::<i32>);
    let mut turn = use_signal(|| None::<i32>);
    let mut forces = use_signal(Vec::<ForceInfo>::new);
    let mut selected = use_signal(|| None::<(i32, i32)>);
    let mut mounted = use_signal(|| false);

    let refresh = use_callback(move |_: ()| {
        spawn(async move {
            if let Ok(s) = rpc::call(&conn, MapStatusRequest).await {
                status.set(Some(s.in_map));
                if s.in_map {
                    if let Ok(g) = rpc::call(&conn, MapGridRequest).await {
                        grid.set((g.width, g.height));
                    }
                    if let Ok(p) = rpc::call(&conn, MapPlacementsRequest).await {
                        placements.set(p.units);
                    }
                    if let Ok(t) = rpc::call(&conn, MapTurnRequest).await {
                        turn.set(Some(t.turn));
                    }
                    if let Ok(r) = rpc::call(&conn, RewindEntriesRequest).await {
                        rewind.set(r.entries);
                    }
                }
            }
        });
    });

    if !mounted() {
        mounted.set(true);
        refresh.call(());
        spawn(async move {
            if let Ok(f) = rpc::call(&conn, GetForcesRequest).await {
                forces.set(f.forces);
            }
        });
    }

    let on_class_change = move |req: SetClassRequest| {
        spawn(async move {
            match rpc::call(&conn, req).await {
                Ok(_) => refresh.call(()),
                Err(e) => toasts.show(format!("Class change failed: {e}")),
            }
        });
    };
    let on_acted = move |req: SetActedRequest| {
        spawn(async move {
            match rpc::call(&conn, req).await {
                Ok(_) => refresh.call(()),
                Err(e) => toasts.show(format!("Could not change acted: {e}")),
            }
        });
    };
    let move_to = move |(force_id, unit_index, x, z): (i32, i32, i32, i32)| {
        spawn(async move {
            match rpc::call(&conn, SetUnitPosRequest { force_id, unit_index, x, z }).await {
                Ok(_) => refresh.call(()),
                Err(e) => toasts.show(format!("Move failed: {e}")),
            }
        });
    };

    let preview = move |index: i32| {
        spawn(async move {
            if rpc::call(&conn, RewindPreviewRequest { index }).await.is_ok() {
                previewing.set(Some(index));
                refresh.call(());
            }
        });
    };
    let commit_rewind = move |_| {
        spawn(async move {
            let _ = rpc::call(&conn, RewindCommitRequest).await;
            previewing.set(None);
            refresh.call(());
        });
    };
    let cancel_rewind = move |_| {
        spawn(async move {
            let _ = rpc::call(&conn, RewindCancelRequest).await;
            previewing.set(None);
            refresh.call(());
        });
    };

    let commit_turn = move |v: i32| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, SetMapTurnRequest { turn: v }).await {
                turn.set(Some(resp.turn));
            }
        });
    };

    let on_move = move |req: MoveUnitRequest| {
        spawn(async move {
            match rpc::call(&conn, req).await {
                Ok(_) => {
                    selected.set(None);
                    refresh.call(());
                }
                Err(e) => toasts.show(format!("Move failed: {e}")),
            }
        });
    };

    let (width, height) = grid();
    let units = placements();
    let unit_at = move |x: i32, z: i32| units.iter().find(|u| u.x == x && u.z == z).cloned();
    let selected_unit = selected().and_then(|(fid, idx)| {
        placements().into_iter().find(|u| u.force_id == fid && u.unit_index == idx)
    });

    rsx! {
        div { class: "flex flex-1 min-h-0",
            if status() == Some(true) {
                div { class: "w-56 shrink-0 bg-gray-900 border-r border-gray-700 flex flex-col min-h-0",
                    div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700 shrink-0",
                        h2 { class: "text-white font-bold text-sm", "Rewind" }
                        if previewing().is_some() {
                            div { class: "flex gap-1 mt-1",
                                button {
                                    class: "flex-1 bg-emerald-600 hover:bg-emerald-500 text-white text-xs rounded px-2 py-0.5",
                                    onclick: commit_rewind,
                                    "Confirm"
                                }
                                button {
                                    class: "flex-1 bg-gray-700 hover:bg-gray-600 text-white text-xs rounded px-2 py-0.5",
                                    onclick: cancel_rewind,
                                    "Cancel"
                                }
                            }
                        }
                    }
                    div { class: "flex-1 overflow-auto",
                        if rewind().is_empty() {
                            p { class: "text-gray-500 text-xs p-3", "No rewind history." }
                        }
                        for e in rewind().into_iter() {
                            {
                                let active = previewing() == Some(e.index);
                                let row_class = if active {
                                    "w-full text-left px-3 py-1 text-xs border-b border-gray-800 bg-indigo-900/60 ring-1 ring-indigo-500"
                                } else {
                                    "w-full text-left px-3 py-1 text-xs border-b border-gray-800 hover:bg-gray-800"
                                };
                                rsx! {
                            button {
                                key: "{e.index}",
                                class: "{row_class}",
                                title: "Preview this point",
                                onclick: {
                                    let index = e.index;
                                    move |_| preview(index)
                                },
                                if e.is_phase_begin {
                                    span { class: "text-indigo-300 font-semibold", "{force_label(e.force)} phase" }
                                } else {
                                    div {
                                        span { class: "text-white", "{e.action}" }
                                        if !e.actor.is_empty() {
                                            span { class: "text-gray-400", " — {e.actor}" }
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
            div { class: "flex flex-col flex-1 min-h-0",
                div { class: "flex items-center gap-3 px-4 py-2 bg-gray-900 border-b border-gray-700 shrink-0",
                    h2 { class: "text-white font-bold text-sm", "Map" }
                    if status() == Some(true) {
                        if let Some(t) = turn() {
                            label { class: "flex items-center gap-1 text-gray-400 text-xs",
                                "Turn"
                                input {
                                    key: "turn-{t}",
                                    r#type: "number",
                                    class: "w-14 px-1 py-0.5 bg-gray-800 text-yellow-300 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center",
                                    value: "{t}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().trim().parse::<i32>() {
                                            if v != t { commit_turn(v); }
                                        }
                                    },
                                }
                            }
                        }
                    }
                    button {
                        class: "text-indigo-300 hover:text-indigo-200 text-xs",
                        onclick: move |_| refresh.call(()),
                        "Refresh"
                    }
                    if status() == Some(true) {
                        button {
                            class: "ml-auto flex items-center gap-1.5 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white text-xs font-medium rounded-md px-3 py-1 shadow-sm transition-colors",
                            title: "Sets the victory game variable, ending the map as a win",
                            onclick: move |_| {
                                spawn(async move {
                                    if rpc::call(&conn, CompleteMapRequest).await.is_ok() {
                                        toasts.show("Map marked as complete. It takes effect after the next turn.");
                                    }
                                });
                            },
                            span { class: "text-sm leading-none", "🏆" }
                            "Complete map"
                        }
                    }
                }
                div {
                    class: "flex-1 min-h-0 overflow-auto p-4",
                    // clicking the map area outside the grid clears the selection. cell clicks
                    // stop propagation below, so this only fires for the empty background
                    onclick: move |_| selected.set(None),
                    match status() {
                        None => rsx! { p { class: "text-gray-400 text-sm", "Checking map state..." } },
                        Some(false) => rsx! {
                            p { class: "text-gray-500 text-sm",
                                "Not in an ongoing battle. Hit Refresh when one has started."
                            }
                        },
                        Some(true) if width == 0 || height == 0 => rsx! {
                            p { class: "text-gray-500 text-sm",
                                "Not in an ongoing battle. Hit Refresh when one has started."
                            }
                        },
                        Some(true) => rsx! {
                            div {
                                class: "inline-flex flex-col gap-px bg-gray-700",
                                // keep grid clicks (select / move-to) from bubbling to the
                                // background deselect handler
                                onclick: |e| e.stop_propagation(),
                                for z in (0..height).rev() {
                                    div { class: "flex gap-px",
                                        for x in 0..width {
                                            {
                                                let cell = unit_at(x, z);
                                                let cell_unit = cell.as_ref().map(|u| (u.force_id, u.unit_index));
                                                let is_sel = cell_unit == selected();
                                                rsx! {
                                                    div {
                                                        class: "w-5 h-5 bg-gray-800 flex items-center justify-center cursor-pointer hover:bg-gray-700",
                                                        onclick: move |_| match cell_unit {
                                                            Some((fid, idx)) if selected() == Some((fid, idx)) => selected.set(None),
                                                            Some((fid, idx)) => selected.set(Some((fid, idx))),
                                                            None => {
                                                                if let Some((fid, idx)) = selected() {
                                                                    move_to((fid, idx, x, z));
                                                                }
                                                            }
                                                        },
                                                        if let Some(u) = cell {
                                                            {
                                                                let ring = if is_sel { "ring-2 ring-white" } else { "" };
                                                                let dim = if u.acted { "opacity-40" } else { "" };
                                                                let color = force_color(u.force_id);
                                                                rsx! {
                                                                    div {
                                                                        class: "w-3.5 h-3.5 rounded-full {color} {ring} {dim}",
                                                                        title: "{u.name} (Lv {u.level})",
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
                            }
                        },
                    }
                }
                if status() == Some(true) {
                    div { class: "h-72 shrink-0 flex flex-col min-h-0 border-t border-gray-700",
                        GlobalsView { temporary_only: true }
                    }
                }
            }
            if let Some(u) = selected_unit {
                ResizablePanel {
                    class: "bg-gray-800 border-l border-gray-700 overflow-auto",
                    default_width: 384.0,
                    div { class: "flex items-center justify-between px-4 py-2 bg-gray-900 border-b border-gray-700",
                        span { class: "text-gray-400 text-xs", "Selected unit" }
                        button {
                            class: "text-gray-400 hover:text-white text-sm",
                            title: "Close (deselect)",
                            onclick: move |_| selected.set(None),
                            "✕"
                        }
                    }
                    UnitInspector {
                        force_id: u.force_id,
                        unit: UnitSummary {
                            index: u.unit_index,
                            name: u.name.clone(),
                            level: u.level,
                            internal_level: u.internal_level,
                            total_level: u.total_level,
                            class_jid: u.class_jid.clone(),
                            acted: u.acted,
                        },
                        classes: catalogs().classes,
                        item_catalog: catalogs().items,
                        force_options: forces(),
                        on_class_change: on_class_change,
                        on_move: on_move,
                        on_acted: on_acted,
                    }
                }
            }
        }
    }
}
