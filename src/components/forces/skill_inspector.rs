use dioxus::prelude::*;

use crate::components::catalog_provider::Catalogs;
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

    let arrow = if open() { "▾" } else { "▸" };

    rsx! {
        div { class: "mt-1",
            button {
                class: "text-gray-400 hover:text-gray-200 text-xs",
                onclick: toggle,
                "{arrow} Skills"
            }
            if open() {
                div { class: "mt-1 pl-2 border-l border-gray-700",
                    match skills() {
                        None => rsx! { p { class: "text-gray-500 text-xs py-1", "Loading skills..." } },
                        Some(list) if list.is_empty() => rsx! {
                            p { class: "text-gray-500 text-xs py-1", "No skills." }
                        },
                        Some(list) => rsx! {
                            for sk in list.into_iter() {
                                div { key: "{sk.source}-{sk.sid}", class: "flex items-center gap-2 py-0.5 text-xs",
                                    span { class: "text-gray-500 w-16 shrink-0", "{sk.source}" }
                                    span { class: "text-white flex-1 truncate", title: "{sk.sid}", "{sk.name}" }
                                    if sk.removable {
                                        button {
                                            class: "text-red-500 hover:text-red-300",
                                            title: "Remove",
                                            onclick: {
                                                let sid = sk.sid.clone();
                                                let source = sk.source.clone();
                                                move |_| remove((sid.clone(), source.clone()))
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        },
                    }
                    div { class: "flex items-center gap-1 mt-1",
                        select {
                            class: "bg-gray-900 text-gray-300 text-xs rounded border border-gray-600 px-1 py-0.5",
                            onchange: move |e| target.set(e.value()),
                            for t in ["Equip", "Private", "Job"] {
                                option { value: "{t}", selected: t == target(), "{t}" }
                            }
                        }
                        select {
                            class: "flex-1 bg-gray-900 text-emerald-300 text-xs rounded border border-gray-600 px-1 py-0.5",
                            onchange: move |e| {
                                let sid = e.value();
                                if !sid.is_empty() { assign(sid); }
                            },
                            option { value: "", selected: true, disabled: true, "+ Assign skill…" }
                            for c in catalogs().skills.iter().filter(|c| match target().as_str() {
                                "Private" => !c.inheritable,
                                "Equip" => c.inheritable,
                                _ => true,
                            }) {
                                option { value: "{c.sid}", "{c.name}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
