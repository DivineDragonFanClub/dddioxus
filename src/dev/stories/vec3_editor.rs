use dioxus::prelude::*;

use crate::components::fields::Vec3Editor;
use crate::dev::{StoryPage, StorySection};
use crate::protocol::Vec3;

#[component]
pub fn DevVec3Editor() -> Element {
    rsx! {
        StoryPage { title: "Vec3Editor",
            StorySection { label: "Identity",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Position",
                        value: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Non-zero values",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Rotation",
                        value: Vec3 { x: 14.25, y: 90.0, z: -7.5 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Unit scale",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Scale",
                        value: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Negative values",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Offset",
                        value: Vec3 { x: -1.5, y: -2.0, z: -3.25 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
        }
    }
}
