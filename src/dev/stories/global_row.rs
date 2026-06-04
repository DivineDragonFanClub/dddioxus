use dioxus::prelude::*;

use crate::components::globals_view::GlobalRow;
use crate::dev::{fixtures, StoryPage, StorySection};
use crate::protocol::GlobalVariable;

#[component]
pub fn DevGlobalRow() -> Element {
    rsx! {
        StoryPage { title: "GlobalRow",
            StorySection { label: "Integer value",
                div { class: "p-3 font-mono text-xs",
                    GlobalRow {
                        variable: fixtures::sample_global_variable(),
                        on_commit: |_| {},
                    }
                }
            }
            StorySection { label: "String value",
                div { class: "p-3 font-mono text-xs",
                    GlobalRow {
                        variable: fixtures::sample_global_variable_string(),
                        on_commit: |_| {},
                    }
                }
            }
            StorySection { label: "Float value",
                div { class: "p-3 font-mono text-xs",
                    GlobalRow {
                        variable: GlobalVariable {
                            name: "G_GameTime".into(),
                            kind: "float".into(),
                            value: "14237.5".into(),
                            temporary: false,
                        },
                        on_commit: |_| {},
                    }
                }
            }
        }
    }
}
