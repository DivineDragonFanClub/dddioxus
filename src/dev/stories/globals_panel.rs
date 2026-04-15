use dioxus::prelude::*;

use crate::components::globals_view::GlobalsPanel;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevGlobalsPanel() -> Element {
    rsx! {
        StoryPage { title: "GlobalsPanel",
            StorySection { label: "Loaded",
                div { class: "h-96 flex",
                    GlobalsPanel {
                        data: Some(Ok(fixtures::globals_loaded())),
                        loading: false,
                        on_refresh: |_| {},
                        on_commit: |_| {},
                    }
                }
            }
            StorySection { label: "Refreshing",
                div { class: "h-96 flex",
                    GlobalsPanel {
                        data: Some(Ok(fixtures::globals_loaded())),
                        loading: true,
                        on_refresh: |_| {},
                        on_commit: |_| {},
                    }
                }
            }
            StorySection { label: "Empty",
                div { class: "h-40 flex",
                    GlobalsPanel {
                        data: Some(Ok(fixtures::globals_empty())),
                        loading: false,
                        on_refresh: |_| {},
                        on_commit: |_| {},
                    }
                }
            }
            StorySection { label: "Initial loading",
                div { class: "h-40 flex",
                    GlobalsPanel {
                        data: None,
                        loading: false,
                        on_refresh: |_| {},
                        on_commit: |_| {},
                    }
                }
            }
            StorySection { label: "Error",
                div { class: "h-40 flex",
                    GlobalsPanel {
                        data: Some(Err("Not connected".into())),
                        loading: false,
                        on_refresh: |_| {},
                        on_commit: |_| {},
                    }
                }
            }
        }
    }
}
