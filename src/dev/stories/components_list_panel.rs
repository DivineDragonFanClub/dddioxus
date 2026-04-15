use dioxus::prelude::*;

use crate::components::components_panel::ComponentsListPanel;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevComponentsListPanel() -> Element {
    let noop = use_callback(|_index: u32| {});

    rsx! {
        StoryPage { title: "ComponentsListPanel",
            StorySection { label: "Loaded",
                div { class: "bg-gray-950",
                    ComponentsListPanel {
                        data: Some(Ok(fixtures::components_loaded())),
                        on_refresh: |_| {},
                        on_toggle: noop,
                    }
                }
            }
            StorySection { label: "Empty",
                div { class: "bg-gray-950",
                    ComponentsListPanel {
                        data: Some(Ok(fixtures::components_empty())),
                        on_refresh: |_| {},
                        on_toggle: noop,
                    }
                }
            }
            StorySection { label: "Loading",
                div { class: "bg-gray-950",
                    ComponentsListPanel {
                        data: None,
                        on_refresh: |_| {},
                        on_toggle: noop,
                    }
                }
            }
            StorySection { label: "Error",
                div { class: "bg-gray-950",
                    ComponentsListPanel {
                        data: Some(Err("Path not found".into())),
                        on_refresh: |_| {},
                        on_toggle: noop,
                    }
                }
            }
        }
    }
}
