use dioxus::prelude::*;

use super::stat_field::StatField;
use crate::components::ui::{EmptyState, StateKind};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{GetStatsRequest, SetStatRequest, StatValue};
use crate::rpc;

#[derive(PartialEq, Clone, Props)]
pub struct StatInspectorProps {
    pub force_id: i32,
    pub unit_index: i32,
    // bumped by the parent after a level/internal change with "grow" on, so we refetch the stats
    pub refresh: u32,
}

#[component]
pub fn StatInspector(props: StatInspectorProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut stats = use_signal(|| None::<Vec<StatValue>>);

    let force_id = props.force_id;
    let unit_index = props.unit_index;

    let fetch = move || {
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetStatsRequest { force_id, unit_index }).await {
                stats.set(Some(resp.stats));
            }
        });
    };

    let mut mounted = use_signal(|| false);
    let mut last_refresh = use_signal(|| props.refresh);
    if !mounted() {
        mounted.set(true);
        fetch();
    } else if last_refresh() != props.refresh {
        last_refresh.set(props.refresh);
        fetch();
    }

    let on_commit = move |(stat_index, value): (i32, i32)| {
        spawn(async move {
            let req = SetStatRequest { force_id, unit_index, stat_index, value };
            if let Ok(resp) = rpc::call(&conn, req).await {
                stats.with_mut(|slot| {
                    if let Some(list) = slot.as_mut() {
                        if let Some(s) = list.iter_mut().find(|s| s.index == stat_index) {
                            s.value = resp.value;
                        }
                    }
                });
            }
        });
    };

    rsx! {
        match stats() {
            Some(list) => rsx! {
                div { class: "flex flex-wrap gap-2 py-1",
                    for stat in list.into_iter() {
                        StatField { key: "{stat.index}", stat, on_commit }
                    }
                }
            },
            None => rsx! {
                EmptyState { kind: StateKind::Loading, message: "Loading stats\u{2026}", compact: true }
            },
        }
    }
}
