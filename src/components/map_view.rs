use dioxus::prelude::*;

use crate::components::catalog_provider::Catalogs;
use crate::components::forces::{force_dot, force_label, UnitInspector};
use crate::components::globals_view::GlobalsView;
use crate::components::resizable_panel::ResizablePanel;
use crate::components::toast::use_toasts;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Card, EditableNumber, EmptyState, ListRow, Page, PanelHeader,
    StateKind, Tone,
};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    CompleteMapRequest, ForceInfo, GetForcesRequest, MapGridRequest, MapPlacementsRequest, MapStatusRequest,
    MapTurnRequest, MapUnit, MoveUnitRequest, RewindCancelRequest, RewindCommitRequest, RewindEntriesRequest,
    RewindEntry, RewindPreviewRequest, SetActedRequest, SetClassRequest, SetMapTurnRequest, SetUnitPosRequest,
    UnitSummary,
};
use crate::rpc;

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
    // side panels start hidden so the map gets the full width, toggle them on from the header
    let mut show_rewind = use_signal(|| false);
    let mut show_vars = use_signal(|| false);

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

    let commit_turn = move |v: i64| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, SetMapTurnRequest { turn: v as i32 }).await {
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
        Page { row: true,
            // rewind sidebar, only shown during an active map and when toggled on
            if status() == Some(true) && show_rewind() {
                Card {
                    class: "w-56 shrink-0",
                    padded: false,
                    header: rsx! {
                        if previewing().is_some() {
                            Button {
                                tone: Tone::Emerald,
                                size: ButtonSize::Sm,
                                onclick: commit_rewind,
                                "Confirm"
                            }
                            Button {
                                tone: Tone::Gray,
                                size: ButtonSize::Sm,
                                onclick: cancel_rewind,
                                "Cancel"
                            }
                        }
                    },
                    title: "Rewind",
                    div { class: "flex-1 overflow-auto",
                        if rewind().is_empty() {
                            EmptyState { kind: StateKind::Empty, message: "No rewind history.", compact: true }
                        }
                        for e in rewind().into_iter() {
                            {
                                let active = previewing() == Some(e.index);
                                rsx! {
                                    ListRow {
                                        key: "{e.index}",
                                        selected: active,
                                        onclick: {
                                            let index = e.index;
                                            move |_| preview(index)
                                        },
                                        if e.is_phase_begin {
                                            span { class: "text-indigo-300 font-semibold text-xs", "{force_label(e.force)} phase" }
                                        } else {
                                            div { class: "min-w-0",
                                                span { class: "text-white text-xs", "{e.action}" }
                                                if !e.actor.is_empty() {
                                                    span { class: "text-gray-400 text-xs", " \u{2014} {e.actor}" }
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

            // center column: the map grid, with the local variables panel below it (gap between them)
            div { class: "flex flex-col flex-1 min-h-0 gap-3",
            Card {
                class: "flex-1 min-h-0",
                padded: false,
                title: "Map",
                header: rsx! {
                    if status() == Some(true) {
                        // panel toggles, a little dashboard menu bar for the side panels
                        Button {
                            tone: if show_rewind() { Tone::Indigo } else { Tone::Gray },
                            variant: if show_rewind() { ButtonVariant::Solid } else { ButtonVariant::Outline },
                            size: ButtonSize::Sm,
                            title: "Toggle the rewind history panel",
                            onclick: move |_| show_rewind.set(!show_rewind()),
                            "Rewind"
                        }
                        Button {
                            tone: if show_vars() { Tone::Indigo } else { Tone::Gray },
                            variant: if show_vars() { ButtonVariant::Solid } else { ButtonVariant::Outline },
                            size: ButtonSize::Sm,
                            title: "Toggle the local variables panel",
                            onclick: move |_| show_vars.set(!show_vars()),
                            "Variables"
                        }
                        if let Some(t) = turn() {
                            span { class: "text-gray-400 text-xs shrink-0 ml-1", "Turn" }
                            EditableNumber {
                                value: t as i64,
                                width: "w-14",
                                on_commit: commit_turn,
                            }
                        }
                        Button {
                            tone: Tone::Emerald,
                            size: ButtonSize::Sm,
                            title: "Sets the victory game variable, ending the map as a win",
                            onclick: move |_| {
                                spawn(async move {
                                    if rpc::call(&conn, CompleteMapRequest).await.is_ok() {
                                        toasts.show("Map marked as complete. It takes effect after the next turn.");
                                    }
                                });
                            },
                            "Complete map"
                        }
                    }
                    Button {
                        size: ButtonSize::Sm,
                        onclick: move |_| refresh.call(()),
                        "Refresh"
                    }
                },
                div {
                    class: "flex-1 min-h-0 overflow-auto p-4",
                    // clicking outside the grid clears the selection; cell clicks stop propagation
                    onclick: move |_| selected.set(None),
                    match status() {
                        None => rsx! { EmptyState { kind: StateKind::Loading, message: "Checking map state\u{2026}" } },
                        Some(false) => rsx! {
                            EmptyState {
                                kind: StateKind::Empty,
                                message: "Not in an ongoing battle. Hit Refresh when one has started.",
                            }
                        },
                        Some(true) if width == 0 || height == 0 => rsx! {
                            EmptyState {
                                kind: StateKind::Empty,
                                message: "Not in an ongoing battle. Hit Refresh when one has started.",
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
                                                                let color = force_dot(u.force_id);
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
            }
            if status() == Some(true) && show_vars() {
                div { class: "h-72 shrink-0 flex flex-col min-h-0",
                    GlobalsView { temporary_only: true }
                }
            }
            }

            // right unit inspector, only shown when a unit is selected
            if let Some(u) = selected_unit {
                ResizablePanel {
                    class: "overflow-auto",
                    default_width: 384.0,
                    div { class: "flex flex-col min-h-0 bg-gray-800/80 border border-gray-700/70 rounded-xl shadow-lg shadow-black/30 overflow-hidden h-full",
                        PanelHeader {
                            title: "Selected unit",
                            actions: rsx! {
                                Button {
                                    tone: Tone::Gray,
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    title: "Close (deselect)",
                                    onclick: move |_| selected.set(None),
                                    "\u{2715}"
                                }
                            },
                        }
                        div { class: "flex-1 overflow-auto",
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
    }
}
