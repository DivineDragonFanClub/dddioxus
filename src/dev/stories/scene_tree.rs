use dioxus::prelude::*;

use crate::components::scene_tree::SceneTree;
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevSceneTree() -> Element {
    let loaded = fixtures::scene_loaded();
    let empty = fixtures::scene_empty();

    rsx! {
        StoryPage { title: "SceneTree",
            StorySection { label: "Populated scene",
                div { class: "h-96 overflow-auto bg-gray-800",
                    SceneTree {
                        scenes: loaded.scenes.clone(),
                        selected_path: None,
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
            StorySection { label: "Selected node",
                div { class: "h-96 overflow-auto bg-gray-800",
                    SceneTree {
                        scenes: loaded.scenes.clone(),
                        selected_path: Some("MainGameScene/World/Units/Alear".into()),
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
            StorySection { label: "Empty scene",
                div { class: "h-40 overflow-auto bg-gray-800",
                    SceneTree {
                        scenes: empty.scenes,
                        selected_path: None,
                        on_select: |_: String| {},
                        on_toggle_active: |_: String| {},
                    }
                }
            }
        }
    }
}
