use dioxus::prelude::*;

use crate::components::procs_view::DescsPanel;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevDescsPanel() -> Element {
    rsx! {
        StoryPage { title: "DescsPanel",
            StorySection { label: "Loaded",
                div { class: "h-80 flex bg-gray-800",
                    DescsPanel {
                        selection: fixtures::sample_selection(),
                        width: 320.0,
                        data: Some(Ok(fixtures::proc_descs_loaded())),
                    }
                }
            }
            StorySection { label: "Empty",
                div { class: "h-40 flex bg-gray-800",
                    DescsPanel {
                        selection: fixtures::sample_selection(),
                        width: 320.0,
                        data: Some(Ok(fixtures::proc_descs_empty())),
                    }
                }
            }
            StorySection { label: "Loading",
                div { class: "h-40 flex bg-gray-800",
                    DescsPanel {
                        selection: fixtures::sample_selection(),
                        width: 320.0,
                        data: None,
                    }
                }
            }
            StorySection { label: "Error",
                div { class: "h-40 flex bg-gray-800",
                    DescsPanel {
                        selection: fixtures::sample_selection(),
                        width: 320.0,
                        data: Some(Err("desc index out of range".into())),
                    }
                }
            }
        }
    }
}
