use dioxus::prelude::*;

use super::bond_inspector::BondInspector;
use super::inventory_inspector::InventoryInspector;
use super::skill_inspector::SkillInspector;
use super::stat_inspector::StatInspector;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    ClassInfo, ForceInfo, ItemCatalogEntry, MoveUnitRequest, SetActedRequest, SetClassRequest, SetInternalLevelRequest,
    SetLevelRequest, UnitSummary,
};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct UnitInspectorProps {
    pub force_id: i32,
    pub unit: UnitSummary,
    pub classes: Vec<ClassInfo>,
    pub item_catalog: Vec<ItemCatalogEntry>,
    pub force_options: Vec<ForceInfo>,
    pub on_class_change: EventHandler<SetClassRequest>,
    pub on_move: EventHandler<MoveUnitRequest>,
    pub on_acted: EventHandler<SetActedRequest>,
}

#[component]
pub fn UnitInspector(props: UnitInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let force_id = props.force_id;
    let unit_index = props.unit.index;
    let on_class_change = props.on_class_change;
    let on_move = props.on_move;
    let on_acted = props.on_acted;
    let acted = props.unit.acted;
    // Dead (4), Lost (5) and Temporary (6) units don't take turns, so the acted toggle is moot
    let show_acted = !matches!(force_id, 4 | 5 | 6);

    // level / internal level live right on App.Unit, so we edit them in the header and keep the
    // displayed values in local signals (seeded from the roster, refreshed from each set response)
    let mut level = use_signal(|| props.unit.level);
    let mut internal = use_signal(|| props.unit.internal_level);
    let mut total = use_signal(|| props.unit.total_level);
    // whether a level/internal edit should regrow the stat block to match
    let mut grow = use_signal(|| false);
    // bumped after a grow so the StatInspector below refetches
    let mut stat_gen = use_signal(|| 0u32);

    let commit_level = move |v: i32| {
        let grow_stats = grow();
        spawn(async move {
            let req = SetLevelRequest { force_id, unit_index, level: v, grow_stats };
            if let Ok(info) = rpc::call(&conn, req).await {
                level.set(info.level);
                internal.set(info.internal_level);
                total.set(info.total_level);
                if grow_stats {
                    stat_gen += 1;
                }
            }
        });
    };

    let commit_internal = move |v: i32| {
        let grow_stats = grow();
        spawn(async move {
            let req = SetInternalLevelRequest { force_id, unit_index, internal_level: v, grow_stats };
            if let Ok(info) = rpc::call(&conn, req).await {
                level.set(info.level);
                internal.set(info.internal_level);
                total.set(info.total_level);
                if grow_stats {
                    stat_gen += 1;
                }
            }
        });
    };

    rsx! {
        div { class: "border-b border-gray-700 px-4 py-3 min-w-0",
            div { class: "flex flex-wrap items-center gap-x-3 gap-y-1 mb-2",
                if show_acted {
                    label {
                        class: "flex items-center gap-1 text-gray-400 text-xs cursor-pointer select-none",
                        title: "Whether the unit has used up its turn",
                        input {
                            key: "{acted}",
                            r#type: "checkbox",
                            class: "accent-indigo-500",
                            checked: acted,
                            onchange: move |e| {
                                on_acted.call(SetActedRequest {
                                    force_id,
                                    unit_index,
                                    acted: e.checked(),
                                });
                            },
                        }
                        "Acted"
                    }
                }
                span { class: "text-white font-semibold text-sm", "{props.unit.name}" }
                IntField { label: "Lv", value: level(), on_commit: commit_level }
                IntField { label: "iLv", value: internal(), on_commit: commit_internal }
                span {
                    class: "text-gray-500 text-xs",
                    title: "Total level (level + internal level), the grow target when 'grow' is on",
                    "= {total}"
                }
                label {
                    class: "flex items-center gap-1 text-gray-400 text-xs cursor-pointer select-none",
                    title: "Regrow the stat block to match the new level when editing Lv / iLv",
                    input {
                        r#type: "checkbox",
                        class: "accent-indigo-500",
                        checked: grow(),
                        onchange: move |e| grow.set(e.checked()),
                    }
                    "grow"
                }
                select {
                    class: "bg-gray-900 text-indigo-300 text-xs rounded border border-gray-600 px-1 py-0.5 focus:border-indigo-500 focus:outline-none",
                    value: "{props.unit.class_jid}",
                    onchange: move |e| {
                        on_class_change.call(SetClassRequest {
                            force_id,
                            unit_index,
                            jid: e.value(),
                        });
                    },
                    for c in props.classes.iter() {
                        option { value: "{c.jid}", selected: c.jid == props.unit.class_jid, "{c.name}" }
                    }
                }
                select {
                    class: "ml-auto bg-gray-900 text-gray-300 text-xs rounded border border-gray-600 px-1 py-0.5 focus:border-indigo-500 focus:outline-none",
                    onchange: move |e| {
                        if let Ok(to) = e.value().parse::<i32>() {
                            on_move.call(MoveUnitRequest {
                                from_force_id: force_id,
                                unit_index,
                                to_force_id: to,
                            });
                        }
                    },
                    option { value: "", selected: true, disabled: true, "Move to…" }
                    for f in props.force_options.iter() {
                        if f.id != force_id {
                            option { value: "{f.id}", "{f.label}" }
                        }
                    }
                }
            }
            StatInspector { force_id, unit_index, refresh: stat_gen() }
            InventoryInspector {
                force_id,
                unit_index,
                item_catalog: props.item_catalog.clone(),
            }
            SkillInspector { force_id, unit_index }
            BondInspector { force_id, unit_index }
        }
    }
}

// compact inline number field for the level row, commits on Enter or blur
#[derive(PartialEq, Clone, Props)]
struct IntFieldProps {
    label: &'static str,
    value: i32,
    on_commit: EventHandler<i32>,
}

#[component]
fn IntField(props: IntFieldProps) -> Element {
    let mut draft = use_signal(|| props.value.to_string());
    // re-seed when the value changes underneath us (e.g. the server clamped it)
    let mut last = use_signal(|| props.value);
    if last() != props.value {
        last.set(props.value);
        draft.set(props.value.to_string());
    }

    let commit = {
        let on_commit = props.on_commit;
        let current = props.value;
        move || {
            if let Ok(v) = draft().trim().parse::<i32>() {
                if v != current {
                    on_commit.call(v);
                }
            }
        }
    };

    rsx! {
        label { class: "flex items-center gap-1 text-gray-400 text-xs",
            "{props.label}"
            input {
                r#type: "number",
                class: "w-12 px-1 py-0.5 bg-gray-900 text-yellow-300 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center text-sm",
                value: "{draft}",
                oninput: move |e| draft.set(e.value()),
                onblur: {
                    let commit = commit.clone();
                    move |_| commit()
                },
                onkeydown: {
                    let commit = commit.clone();
                    move |e| {
                        if e.key() == Key::Enter {
                            commit();
                        }
                    }
                },
            }
        }
    }
}
