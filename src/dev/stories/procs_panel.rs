use dioxus::prelude::*;

use crate::components::procs_view::{ProcsPanel, Selection};
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevProcsPanel() -> Element {
    let noop = use_callback(|_sel: Selection| {});

    rsx! {
        StoryPage { title: "ProcsPanel",
            StorySection { label: "Loaded, nothing selected",
                div { class: "h-[32rem] flex",
                    ProcsPanel {
                        data: Some(Ok(fixtures::proc_tree_loaded())),
                        loading: false,
                        selected: None,
                        descs_data: None,
                        on_refresh: |_| {},
                        on_select: noop,
                    }
                }
            }
            StorySection { label: "Loaded with selection + descs",
                div { class: "h-[32rem] flex",
                    ProcsPanel {
                        data: Some(Ok(fixtures::proc_tree_loaded())),
                        loading: false,
                        selected: Some(fixtures::sample_selection()),
                        descs_data: Some(Ok(fixtures::proc_descs_loaded())),
                        on_refresh: |_| {},
                        on_select: noop,
                    }
                }
            }
            StorySection { label: "Loading",
                div { class: "h-40 flex",
                    ProcsPanel {
                        data: None,
                        loading: false,
                        selected: None,
                        descs_data: None,
                        on_refresh: |_| {},
                        on_select: noop,
                    }
                }
            }
            StorySection { label: "Error",
                div { class: "h-40 flex",
                    ProcsPanel {
                        data: Some(Err("proc tree unavailable".into())),
                        loading: false,
                        selected: None,
                        descs_data: None,
                        on_refresh: |_| {},
                        on_select: noop,
                    }
                }
            }
        }
    }
}
