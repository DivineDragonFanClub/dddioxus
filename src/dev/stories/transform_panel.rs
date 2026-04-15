use dioxus::prelude::*;

use crate::components::transform_inspector::TransformPanel;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevTransformPanel() -> Element {
    rsx! {
        StoryPage { title: "TransformPanel",
            StorySection { label: "Identity",
                div { class: "bg-gray-950",
                    TransformPanel {
                        data: Some(Ok(fixtures::transform_identity())),
                        on_refresh: |_| {},
                        on_change: |_| {},
                    }
                }
            }
            StorySection { label: "Non-zero",
                div { class: "bg-gray-950",
                    TransformPanel {
                        data: Some(Ok(fixtures::transform_nonzero())),
                        on_refresh: |_| {},
                        on_change: |_| {},
                    }
                }
            }
            StorySection { label: "Loading",
                div { class: "bg-gray-950",
                    TransformPanel {
                        data: None,
                        on_refresh: |_| {},
                        on_change: |_| {},
                    }
                }
            }
            StorySection { label: "Error",
                div { class: "bg-gray-950",
                    TransformPanel {
                        data: Some(Err("Transform missing".into())),
                        on_refresh: |_| {},
                        on_change: |_| {},
                    }
                }
            }
        }
    }
}
