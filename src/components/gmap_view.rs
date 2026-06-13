use dioxus::prelude::*;

use crate::components::toast::use_toasts;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    DespawnEncounterRequest, GetGmapNodesRequest, GetGmapNodesResponse, SetNodeStateRequest,
    SpawnEncounterRequest,
};
use crate::rpc;

// the GmapSpot.State values, in the same order the game uses them. the reserve_* ones are
// mid-animation transition states, the settled ones (Hidden / Unlocked / Locked / Broken) are
// what you normally want
const STATES: &[(i32, &str)] = &[
    (0, "Reserve hide"),
    (1, "Hidden"),
    (2, "Reserve active"),
    (3, "Unlocked"),
    (4, "Reserve lock"),
    (5, "Locked"),
    (6, "Reserve broken"),
    (7, "Broken"),
    (8, "Searchable"),
];

fn rare_label(rare: i32) -> Option<&'static str> {
    match rare {
        1 => Some("EXP"),
        2 => Some("Gold"),
        _ => None,
    }
}

#[component]
pub fn GmapView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let toasts = use_toasts();
    let mut data = use_signal(|| None::<GetGmapNodesResponse>);
    let mut loading = use_signal(|| false);
    let mut search = use_signal(String::new);
    let mut mounted = use_signal(|| false);

    let refresh = use_callback(move |_: ()| {
        if loading() {
            return;
        }
        loading.set(true);
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetGmapNodesRequest).await {
                data.set(Some(resp));
            }
            loading.set(false);
        });
    });

    if !mounted() {
        mounted.set(true);
        refresh.call(());
    }

    let set_state = move |(cid, state): (String, i32)| {
        spawn(async move {
            if rpc::call(&conn, SetNodeStateRequest { cid, state }).await.is_ok() {
                refresh.call(());
            } else {
                toasts.show("Could not change the node state.");
            }
        });
    };

    let spawn_encounter = move |cid: String| {
        spawn(async move {
            match rpc::call(&conn, SpawnEncounterRequest { cid }).await {
                Ok(_) => {
                    toasts.show("Encounter spawned on the node.");
                    refresh.call(());
                }
                Err(e) => toasts.show(format!("Spawn failed: {e}")),
            }
        });
    };

    let despawn_encounter = move |cid: String| {
        spawn(async move {
            match rpc::call(&conn, DespawnEncounterRequest { cid }).await {
                Ok(_) => {
                    toasts.show("Encounter removed from the node.");
                    refresh.call(());
                }
                Err(e) => toasts.show(format!("Despawn failed: {e}")),
            }
        });
    };

    let unlock_all = move |_| {
        // reuse the per-node set_node_state command over every loaded node, no dedicated handler.
        // state 3 = active = unlocked
        let cids: Vec<String> = match data() {
            Some(resp) => resp.nodes.iter().map(|n| n.cid.clone()).collect(),
            None => Vec::new(),
        };
        spawn(async move {
            let mut count = 0;
            for cid in cids {
                if rpc::call(&conn, SetNodeStateRequest { cid, state: 3 }).await.is_ok() {
                    count += 1;
                }
            }
            toasts.show(format!("Unlocked {count} nodes."));
            refresh.call(());
        });
    };

    rsx! {
        div {
            "data-component": "GmapView",
            class: "flex flex-col flex-1 min-h-0",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700 shrink-0",
                h2 { class: "text-white font-bold text-sm shrink-0", "Gmap" }
                input {
                    class: "ml-3 flex-1 px-3 py-1 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-sm",
                    placeholder: "Filter nodes...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button {
                    class: "text-white bg-emerald-600 border-0 py-1 px-3 focus:outline-none hover:bg-emerald-500 rounded text-sm shrink-0",
                    onclick: unlock_all,
                    "Unlock all"
                }
                button {
                    class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: loading(),
                    onclick: move |_| refresh.call(()),
                    if loading() { "Refreshing..." } else { "Refresh" }
                }
            }
            div { class: "flex-1 overflow-auto bg-gray-800 p-4 text-sm",
                match data() {
                    None => rsx! { p { class: "text-gray-400", "Loading nodes..." } },
                    Some(resp) if !resp.available => rsx! {
                        p { class: "text-gray-500",
                            "World map not active. Open the world map and hit Refresh."
                        }
                    },
                    Some(resp) => {
                        let query = search().to_lowercase();
                        let nodes: Vec<_> = resp.nodes.into_iter()
                            .filter(|n| query.is_empty()
                                || n.name.to_lowercase().contains(&query)
                                || n.chapter.to_lowercase().contains(&query)
                                || n.cid.to_lowercase().contains(&query))
                            .collect();
                        let total = nodes.len();
                        rsx! {
                            p { class: "text-gray-500 mb-2 font-mono text-xs", "{total} nodes" }
                            for node in nodes.into_iter() {
                                div {
                                    key: "{node.cid}",
                                    class: "flex items-center gap-3 py-1.5 px-2 rounded hover:bg-gray-700",
                                    div { class: "flex-1 min-w-0",
                                        p { class: "text-white truncate", title: "{node.cid}", "{node.name}" }
                                        p { class: "text-gray-500 text-xs truncate",
                                            "{node.chapter} "
                                            span { class: "font-mono text-gray-600", "{node.cid}" }
                                        }
                                    }
                                    select {
                                        class: "bg-gray-900 text-gray-200 text-xs rounded border border-gray-600 px-2 py-1 shrink-0",
                                        onchange: {
                                            let cid = node.cid.clone();
                                            move |e: Event<FormData>| {
                                                if let Ok(state) = e.value().parse::<i32>() {
                                                    set_state((cid.clone(), state));
                                                }
                                            }
                                        },
                                        for (value, label) in STATES.iter() {
                                            option {
                                                value: "{value}",
                                                selected: *value == node.state,
                                                "{label}"
                                            }
                                        }
                                    }
                                    div { class: "w-44 flex items-center justify-end gap-2 shrink-0",
                                        if node.has_encounter {
                                            span { class: "text-amber-300 text-xs",
                                                if let Some(enc) = node.encounter.as_ref() {
                                                    if let Some(rare) = rare_label(enc.rare) {
                                                        "⚔ Rank {enc.rank} ({rare})"
                                                    } else {
                                                        "⚔ Rank {enc.rank}"
                                                    }
                                                } else {
                                                    "⚔ Encounter"
                                                }
                                            }
                                            button {
                                                class: "text-red-400 hover:text-red-300 text-xs border border-red-500/40 rounded px-2 py-0.5",
                                                onclick: {
                                                    let cid = node.cid.clone();
                                                    move |_| despawn_encounter(cid.clone())
                                                },
                                                "Despawn"
                                            }
                                        } else {
                                            button {
                                                class: "text-emerald-300 hover:text-emerald-200 text-xs border border-emerald-500/40 rounded px-2 py-0.5",
                                                onclick: {
                                                    let cid = node.cid.clone();
                                                    move |_| spawn_encounter(cid.clone())
                                                },
                                                "Spawn"
                                            }
                                        }
                                    }
                                }
                            }
                            if total == 0 {
                                p { class: "text-gray-500 italic", "No matching nodes" }
                            }
                        }
                    }
                }
            }
        }
    }
}
