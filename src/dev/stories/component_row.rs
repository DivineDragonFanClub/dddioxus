use dioxus::prelude::*;

use crate::components::components_panel::ComponentRow;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevComponentRow() -> Element {
    let noop = use_callback(|_index: u32| {});

    rsx! {
        StoryPage { title: "ComponentRow",
            StorySection { label: "Enabled",
                div { class: "p-3 font-mono text-xs",
                    ComponentRow {
                        component: fixtures::sample_component_enabled(),
                        on_toggle: noop,
                    }
                }
            }
            StorySection { label: "Disabled",
                div { class: "p-3 font-mono text-xs",
                    ComponentRow {
                        component: fixtures::sample_component_disabled(),
                        on_toggle: noop,
                    }
                }
            }
            StorySection { label: "Unknown (Transform-like)",
                div { class: "p-3 font-mono text-xs",
                    ComponentRow {
                        component: fixtures::sample_component_unknown(),
                        on_toggle: noop,
                    }
                }
            }
        }
    }
}
