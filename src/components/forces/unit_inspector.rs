use dioxus::prelude::*;

use super::bond_inspector::BondInspector;
use super::inventory_inspector::InventoryInspector;
use super::skill_inspector::SkillInspector;
use super::stat_inspector::StatInspector;
use crate::components::ui::{Checkbox, EditableNumber, Field, Select, SelectSize};
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

    let commit_level = move |v: i64| {
        let v = v as i32;
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

    let commit_internal = move |v: i64| {
        let v = v as i32;
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
        div { class: "border-b border-gray-700/70 px-4 py-3 min-w-0",
            // title row: name on the left, the turn-used toggle on the right
            div { class: "flex items-center justify-between gap-2 mb-2",
                span { class: "text-white font-semibold text-sm truncate", "{props.unit.name}" }
                if show_acted {
                    Checkbox {
                        checked: acted,
                        label: "Acted",
                        title: "Whether the unit has used up its turn",
                        on_toggle: move |v| {
                            on_acted.call(SetActedRequest { force_id, unit_index, acted: v });
                        },
                    }
                }
            }
            // stacked labeled fields so the dropdowns never collide when the panel is narrow
            div { class: "space-y-1.5 mb-2",
                Field { label: "Level", label_width: "w-16",
                    EditableNumber {
                        value: level() as i64,
                        width: "w-14",
                        on_commit: commit_level,
                    }
                    Checkbox {
                        checked: grow(),
                        label: "grow",
                        title: "Regrow the stat block to match the new level when editing Lv / iLv",
                        on_toggle: move |v| grow.set(v),
                    }
                }
                Field { label: "Internal", label_width: "w-16",
                    EditableNumber {
                        value: internal() as i64,
                        width: "w-14",
                        on_commit: commit_internal,
                    }
                    span {
                        class: "text-gray-500 text-xs",
                        title: "Total level (level + internal level), the grow target when 'grow' is on",
                        "total {total}"
                    }
                }
                Field { label: "Class", label_width: "w-16",
                    Select {
                        size: SelectSize::Sm,
                        class: "w-full",
                        on_change: move |v: String| {
                            on_class_change.call(SetClassRequest { force_id, unit_index, jid: v });
                        },
                        for c in props.classes.iter() {
                            option { value: "{c.jid}", selected: c.jid == props.unit.class_jid, "{c.name}" }
                        }
                    }
                }
                Field { label: "Move to", label_width: "w-16",
                    Select {
                        size: SelectSize::Sm,
                        class: "w-full",
                        on_change: move |v: String| {
                            if let Ok(to) = v.parse::<i32>() {
                                on_move.call(MoveUnitRequest {
                                    from_force_id: force_id,
                                    unit_index,
                                    to_force_id: to,
                                });
                            }
                        },
                        option { value: "", selected: true, disabled: true, "Move to\u{2026}" }
                        for f in props.force_options.iter() {
                            if f.id != force_id {
                                option { value: "{f.id}", "{f.label}" }
                            }
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
