use dioxus::prelude::*;

use crate::components::scene_view::ScenePanel;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevScenePanel() -> Element {
    rsx! {
        StoryPage { title: "ScenePanel",
            StorySection { label: "Loaded, nothing selected",
                div { class: "h-96 flex",
                    ScenePanel {
                        data: Some(Ok(fixtures::scene_loaded())),
                        loading: false,
                        selected_path: None,
                        on_refresh: |_| {},
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
            StorySection { label: "Loaded with selection",
                div { class: "h-96 flex",
                    ScenePanel {
                        data: Some(Ok(fixtures::scene_loaded())),
                        loading: false,
                        selected_path: Some("MainGameScene/World/Units/Alear".into()),
                        on_refresh: |_| {},
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
            StorySection { label: "Refreshing",
                div { class: "h-96 flex",
                    ScenePanel {
                        data: Some(Ok(fixtures::scene_loaded())),
                        loading: true,
                        selected_path: None,
                        on_refresh: |_| {},
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
            StorySection { label: "Initial loading",
                div { class: "h-40 flex",
                    ScenePanel {
                        data: None,
                        loading: false,
                        selected_path: None,
                        on_refresh: |_| {},
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
            StorySection { label: "Error",
                div { class: "h-40 flex",
                    ScenePanel {
                        data: Some(Err("Not connected".into())),
                        loading: false,
                        selected_path: None,
                        on_refresh: |_| {},
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
        }
    }
}
