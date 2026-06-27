use dioxus::prelude::*;

use super::icon_src;
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
        // isolate so the face's negative z-index stays behind this card's content only
        div { class: "relative isolate overflow-hidden border-b border-gray-700/70 px-4 py-3 min-w-0",
            // face thumbnail tucked behind the name in the top-left corner, faded out
            // toward the bottom-right so the fields below stay readable
            if !props.unit.face.is_empty() {
                img {
                    class: "absolute -top-1 -left-2 h-20 w-auto object-contain opacity-25 pointer-events-none select-none",
                    style: "z-index: -1; mask-image: linear-gradient(to bottom right, rgba(0,0,0,0.95), transparent 70%); -webkit-mask-image: linear-gradient(to bottom right, rgba(0,0,0,0.95), transparent 70%);",
                    src: "/sprite/face/{props.unit.face}.png",
                }
            }
            // header: chibi on the left, then name (top) and the class + level line
            div { class: "flex items-center gap-3 mb-2",
                if let Some(src) = icon_src(&props.unit.icon, &props.unit.icon_png) {
                    img { class: "w-16 h-16 object-contain shrink-0", src: "{src}" }
                }
                div { class: "flex-1 min-w-0",
                    // name on top, turn-used toggle on the right
                    div { class: "flex items-center justify-between gap-2 mb-1",
                        span { class: "text-white font-semibold text-base truncate", "{props.unit.name}" }
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
                    // class dropdown then the level / internal level readout
                    div { class: "flex items-center gap-2",
                        Select {
                            size: SelectSize::Sm,
                            class: "flex-1 min-w-0",
                            on_change: move |v: String| {
                                on_class_change.call(SetClassRequest { force_id, unit_index, jid: v });
                            },
                            for c in props.classes.iter() {
                                option { value: "{c.jid}", selected: c.jid == props.unit.class_jid, "{c.name}" }
                            }
                        }
                        div { class: "flex items-center gap-1.5 shrink-0",
                            span { class: "text-gray-200 text-xs font-bold", "Lvl" }
                            EditableNumber { value: level() as i64, width: "w-10", on_commit: commit_level }
                            span { class: "text-gray-500 text-xs", "/" }
                            EditableNumber { value: internal() as i64, width: "w-10", on_commit: commit_internal }
                            Checkbox {
                                checked: grow(),
                                label: "grow",
                                title: "Regrow the stat block to match the new level when editing Lv / iLv",
                                on_toggle: move |v| grow.set(v),
                            }
                        }
                    }
                }
            }
            // stacked labeled fields so the dropdowns never collide when the panel is narrow
            div { class: "space-y-1.5 mb-2",
                Field { label: "Position", label_width: "w-16",
                    span {
                        class: "text-gray-300 text-xs font-mono",
                        title: "Current tile on the map grid (column, row)",
                        "{props.unit.x}, {props.unit.z}"
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
