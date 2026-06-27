use dioxus::prelude::*;

use super::{IconPicker, PickerOption, SpriteImg};
use crate::components::catalog_provider::Catalogs;
use crate::components::ui::{Button, ButtonSize, ButtonVariant, EmptyState, ListRow, Select, SelectSize, StateKind, Tone};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    AddSkillRequest, GetPersonSkillsRequest, GetUnitSkillsRequest, RemoveSkillRequest, SkillInfo,
};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct SkillInspectorProps {
    pub force_id: i32,
    pub unit_index: i32,
}

#[component]
pub fn SkillInspector(props: SkillInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let catalogs = use_context::<Signal<Catalogs>>();
    let mut open = use_signal(|| false);
    let mut skills = use_signal(|| None::<Vec<SkillInfo>>);
    let mut loaded = use_signal(|| false);
    let mut target = use_signal(|| "Equip".to_string());

    let force_id = props.force_id;
    let unit_index = props.unit_index;

    let load = use_callback(move |_: ()| {
        spawn(async move {
            let mut combined = Vec::new();
            if let Ok(resp) = rpc::call(&conn, GetPersonSkillsRequest { force_id, unit_index }).await {
                combined.extend(resp.skills);
            }
            if let Ok(resp) = rpc::call(&conn, GetUnitSkillsRequest { force_id, unit_index }).await {
                combined.extend(resp.skills);
            }
            skills.set(Some(combined));
        });
    });

    let toggle = move |_| {
        let now_open = !open();
        open.set(now_open);
        if now_open && !loaded() {
            loaded.set(true);
            load.call(());
        }
    };

    let remove = move |(sid, source): (String, String)| {
        spawn(async move {
            if rpc::call(&conn, RemoveSkillRequest { force_id, unit_index, sid, source }).await.is_ok() {
                load.call(());
            }
        });
    };

    let assign = move |sid: String| {
        let target = target();
        spawn(async move {
            if rpc::call(&conn, AddSkillRequest { force_id, unit_index, sid, target }).await.is_ok() {
                load.call(());
            }
        });
    };

    rsx! {
        div { class: "mt-1",
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                onclick: toggle,
                if open() { "\u{25BE} Skills" } else { "\u{25B8} Skills" }
            }
            if open() {
                div { class: "mt-1 pl-2 border-l border-gray-700",
                    match skills() {
                        None => rsx! {
                            EmptyState { kind: StateKind::Loading, message: "Loading skills\u{2026}", compact: true }
                        },
                        Some(list) if list.is_empty() => rsx! {
                            EmptyState { kind: StateKind::Empty, message: "No skills.", compact: true }
                        },
                        Some(list) => rsx! {
                            for sk in list.into_iter() {
                                ListRow {
                                    key: "{sk.source}-{sk.sid}",
                                    if !sk.icon.is_empty() {
                                        SpriteImg { src: "/sprite/skill/{sk.icon}.png", class: "w-6 h-6 object-contain shrink-0" }
                                    }
                                    span { class: "text-gray-500 w-16 shrink-0 text-xs", "{sk.source}" }
                                    span { class: "text-white flex-1 truncate text-xs", title: "{sk.sid}", "{sk.name}" }
                                    if sk.removable {
                                        Button {
                                            tone: Tone::Red,
                                            variant: ButtonVariant::Ghost,
                                            size: ButtonSize::Sm,
                                            title: "Remove",
                                            onclick: {
                                                let sid = sk.sid.clone();
                                                let source = sk.source.clone();
                                                move |_| remove((sid.clone(), source.clone()))
                                            },
                                            "\u{2715}"
                                        }
                                    }
                                }
                            }
                        },
                    }
                    {
                        // a skill only becomes a *visible* equipped skill if it's inheritable
                        // and a slot is free (the game caps equipped skills at 2; extras just
                        // go to the hidden pool). so for Equip we offer only what'll actually show.
                        let loaded = skills().unwrap_or_default();
                        let equipped: Vec<String> = loaded.iter().filter(|s| s.source == "Equip").map(|s| s.sid.clone()).collect();
                        let equip_full = target() == "Equip" && equipped.len() >= 2;
                        rsx! {
                            div { class: "flex items-center gap-1 mt-1",
                                Select {
                                    size: SelectSize::Sm,
                                    on_change: move |v| target.set(v),
                                    for t in ["Equip", "Private", "Job"] {
                                        option { value: "{t}", selected: t == target(), "{t}" }
                                    }
                                }
                                if equip_full {
                                    span { class: "flex-1 text-gray-500 text-xs px-1", "Equip slots full (2/2) \u{2014} remove one to add" }
                                } else {
                                    IconPicker {
                                        placeholder: "+ Assign skill\u{2026}".to_string(),
                                        options: catalogs().skills.iter().filter(|c| match target().as_str() {
                                            "Private" => !c.inheritable,
                                            "Equip" => c.inheritable && !equipped.iter().any(|e| e == &c.sid),
                                            _ => true,
                                        }).map(|c| PickerOption {
                                            value: c.sid.clone(),
                                            label: c.name.clone(),
                                            icon: if c.icon.is_empty() { None } else { Some(format!("/sprite/skill/{}.png", c.icon)) },
                                        }).collect::<Vec<_>>(),
                                        on_select: move |v: String| assign(v),
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
