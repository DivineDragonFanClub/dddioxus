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
            StorySection { label: "Whole numbers",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Whole",
                        value: Vec3 { x: 1.0, y: 2.0, z: 3.0 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Small fractions",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Tiny",
                        value: Vec3 { x: 0.0001, y: 0.001, z: 0.01 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Long decimals (rounded to 4 places)",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Math",
                        value: Vec3 { x: 3.141_592_65, y: 2.718_28, z: 1.414_213_56 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Large values",
                div { class: "p-3",
                    Vec3Editor {
                        label: "World",
                        value: Vec3 { x: 14237.5, y: 999999.0, z: -12500.25 },
                        on_change: |_: Vec3| {},
                    }
                }
            }
            StorySection { label: "Rotation (wraps 0..360)",
                div { class: "p-3",
                    Vec3Editor {
                        label: "Euler",
                        value: Vec3 { x: 45.0, y: 180.0, z: 270.0 },
                        wrap: Some(360.0),
                        on_change: |_: Vec3| {},
                    }
                }
            }
        }
    }
}
