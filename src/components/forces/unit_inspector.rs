use dioxus::prelude::*;

use super::bond_inspector::BondInspector;
use super::inventory_inspector::InventoryInspector;
use super::skill_inspector::SkillInspector;
use super::stat_inspector::StatInspector;
use crate::protocol::{
    ClassInfo, ForceInfo, ItemCatalogEntry, MoveUnitRequest, SetActedRequest, SetClassRequest, UnitSummary,
};

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
    let force_id = props.force_id;
    let unit_index = props.unit.index;
    let on_class_change = props.on_class_change;
    let on_move = props.on_move;
    let on_acted = props.on_acted;
    let acted = props.unit.acted;
    rsx! {
        div { class: "border-b border-gray-700 px-4 py-3 min-w-0",
            div { class: "flex flex-wrap items-center gap-x-3 gap-y-1 mb-2",
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
                span { class: "text-white font-semibold text-sm", "{props.unit.name}" }
                span { class: "text-gray-400 text-xs", "Lv {props.unit.level}" }
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
            StatInspector { force_id, unit_index }
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
