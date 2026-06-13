use dioxus::prelude::*;

use crate::components::toast::use_toasts;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Card, EmptyState, ListRow, SearchField, SectionLabel, Select,
    SelectSize, StateKind, Tone,
};
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
        Card {
            class: "flex-1",
            padded: false,
            header: rsx! {
                    SearchField {
                        value: search(),
                        placeholder: "Filter nodes\u{2026}",
                        class: "w-56",
                        on_input: move |v| search.set(v),
                    }
                    Button {
                        tone: Tone::Emerald,
                        onclick: unlock_all,
                        "Unlock all"
                    }
                    Button {
                        disabled: loading(),
                        onclick: move |_| refresh.call(()),
                        if loading() { "Refreshing\u{2026}" } else { "Refresh" }
                    }
                },
                div { class: "p-3",
                    match data() {
                        None => rsx! { EmptyState { kind: StateKind::Loading, message: "Loading nodes\u{2026}" } },
                        Some(resp) if !resp.available => rsx! {
                            EmptyState {
                                kind: StateKind::Empty,
                                message: "World map not active. Open the world map and hit Refresh.",
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
                                SectionLabel { label: "{total} nodes", class: "mb-2" }
                                for node in nodes.into_iter() {
                                    ListRow {
                                        key: "{node.cid}",
                                        div { class: "flex-1 min-w-0",
                                            p { class: "text-white truncate", title: "{node.cid}", "{node.name}" }
                                            p { class: "text-gray-500 text-xs truncate",
                                                "{node.chapter} "
                                                span { class: "font-mono text-gray-600", "{node.cid}" }
                                            }
                                        }
                                        Select {
                                            size: SelectSize::Sm,
                                            class: "shrink-0",
                                            title: "Node state",
                                            on_change: {
                                                let cid = node.cid.clone();
                                                move |v: String| {
                                                    if let Ok(state) = v.parse::<i32>() {
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
                                                span { class: "text-amber-300 text-xs font-medium",
                                                    if let Some(enc) = node.encounter.as_ref() {
                                                        if let Some(rare) = rare_label(enc.rare) {
                                                            "Rank {enc.rank} ({rare})"
                                                        } else {
                                                            "Rank {enc.rank}"
                                                        }
                                                    } else {
                                                        "Encounter"
                                                    }
                                                }
                                                Button {
                                                    tone: Tone::Red,
                                                    variant: ButtonVariant::Outline,
                                                    size: ButtonSize::Sm,
                                                    onclick: {
                                                        let cid = node.cid.clone();
                                                        move |_| despawn_encounter(cid.clone())
                                                    },
                                                    "Despawn"
                                                }
                                            } else {
                                                Button {
                                                    tone: Tone::Emerald,
                                                    variant: ButtonVariant::Outline,
                                                    size: ButtonSize::Sm,
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
                                    EmptyState { kind: StateKind::Empty, message: "No matching nodes" }
                                }
                            }
                        }
                    }
                }
            }
    }
}
