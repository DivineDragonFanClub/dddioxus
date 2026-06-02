use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    ClassInfo, ForceInfo, GetClassesRequest, GetForceUnitsRequest, GetForcesRequest,
    SetUnitClassRequest, SetUnitStatRequest, StatValue, UnitInfo,
};
use crate::rpc;

#[component]
pub fn ForceView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut forces = use_signal(|| None::<Result<Vec<ForceInfo>, String>>);
    let mut selected = use_signal(|| None::<i32>);
    let mut units = use_signal(|| None::<Result<Vec<UnitInfo>, String>>);
    let mut units_loading = use_signal(|| false);
    let mut classes = use_signal(Vec::<ClassInfo>::new);
    let mut mounted = use_signal(|| false);

    if !mounted() {
        mounted.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetForcesRequest).await.map(|r| r.forces);
            forces.set(Some(result));
        });
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetClassesRequest).await {
                classes.set(resp.classes);
            }
        });
    }

    let select_force = use_callback(move |id: i32| {
        selected.set(Some(id));
        units.set(None);
        units_loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetForceUnitsRequest { force_id: id })
                .await
                .map(|r| r.units);
            units.set(Some(result));
            units_loading.set(false);
        });
    });

    // Optimistically patch the edited stat with whatever the server read back.
    let on_commit = move |req: SetUnitStatRequest| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, req.clone()).await {
                units.with_mut(|slot| {
                    if let Some(Ok(list)) = slot.as_mut() {
                        if let Some(u) = list.iter_mut().find(|u| u.index == req.unit_index) {
                            if let Some(s) = u.stats.iter_mut().find(|s| s.index == req.stat_index) {
                                s.value = resp.value;
                            }
                        }
                    }
                });
            }
        });
    };

    let on_class_change = move |req: SetUnitClassRequest| {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, req.clone()).await {
                units.with_mut(|slot| {
                    if let Some(Ok(list)) = slot.as_mut() {
                        if let Some(u) = list.iter_mut().find(|u| u.index == req.unit_index) {
                            u.class = resp.class;
                            u.class_jid = resp.class_jid;
                        }
                    }
                });
            }
        });
    };

    rsx! {
        div { class: "flex h-full",
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
                classes: classes(),
                on_commit: on_commit,
                on_class_change: on_class_change,
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
    pub units: Option<Result<Vec<UnitInfo>, String>>,
    pub force_id: Option<i32>,
    pub loading: bool,
    pub classes: Vec<ClassInfo>,
    pub on_commit: EventHandler<SetUnitStatRequest>,
    pub on_class_change: EventHandler<SetUnitClassRequest>,
}

#[component]
pub fn UnitsPanel(props: UnitsPanelProps) -> Element {
    rsx! {
        div { class: "flex-1 overflow-auto bg-gray-800",
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
                            UnitCard {
                                key: "{u.index}",
                                force_id: props.force_id.unwrap_or(0),
                                unit: u,
                                classes: props.classes.clone(),
                                on_commit: props.on_commit,
                                on_class_change: props.on_class_change,
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

#[derive(PartialEq, Clone, Props)]
pub struct UnitCardProps {
    pub force_id: i32,
    pub unit: UnitInfo,
    pub classes: Vec<ClassInfo>,
    pub on_commit: EventHandler<SetUnitStatRequest>,
    pub on_class_change: EventHandler<SetUnitClassRequest>,
}

#[component]
pub fn UnitCard(props: UnitCardProps) -> Element {
    let force_id = props.force_id;
    let unit_index = props.unit.index;
    let on_class_change = props.on_class_change;
    rsx! {
        div { class: "border-b border-gray-700 px-4 py-3",
            div { class: "flex items-center gap-3 mb-2",
                span { class: "text-white font-semibold text-sm", "{props.unit.name}" }
                span { class: "text-gray-400 text-xs", "Lv {props.unit.level}" }
                select {
                    class: "bg-gray-900 text-indigo-300 text-xs rounded border border-gray-600 px-1 py-0.5 focus:border-indigo-500 focus:outline-none",
                    value: "{props.unit.class_jid}",
                    onchange: move |e| {
                        on_class_change.call(SetUnitClassRequest {
                            force_id,
                            unit_index,
                            jid: e.value(),
                        });
                    },
                    for c in props.classes.iter() {
                        option { value: "{c.jid}", selected: c.jid == props.unit.class_jid, "{c.name}" }
                    }
                }
            }
            div { class: "flex flex-wrap gap-2",
                for stat in props.unit.stats.clone().into_iter() {
                    StatField {
                        key: "{stat.index}",
                        force_id: props.force_id,
                        unit_index: props.unit.index,
                        stat,
                        on_commit: props.on_commit,
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct StatFieldProps {
    pub force_id: i32,
    pub unit_index: i32,
    pub stat: StatValue,
    pub on_commit: EventHandler<SetUnitStatRequest>,
}

#[component]
pub fn StatField(props: StatFieldProps) -> Element {
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.stat.value.to_string());

    let commit = {
        let on_commit = props.on_commit;
        let force_id = props.force_id;
        let unit_index = props.unit_index;
        let stat_index = props.stat.index;
        let current = props.stat.value;
        move || {
            editing.set(false);
            if let Ok(v) = draft().trim().parse::<i32>() {
                if v != current {
                    on_commit.call(SetUnitStatRequest { force_id, unit_index, stat_index, value: v });
                }
            }
        }
    };

    rsx! {
        div { class: "flex flex-col items-center w-12",
            span { class: "text-gray-400 text-[10px] uppercase tracking-wide", "{props.stat.label}" }
            if editing() {
                input {
                    r#type: "number",
                    class: "w-12 px-1 py-0.5 bg-gray-900 text-yellow-300 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center text-sm",
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
                    class: "w-12 text-center text-yellow-300 text-sm cursor-text hover:bg-gray-900 rounded",
                    onclick: {
                        let value = props.stat.value;
                        move |_| {
                            draft.set(value.to_string());
                            editing.set(true);
                        }
                    },
                    "{props.stat.value}"
                }
            }
        }
    }
}
