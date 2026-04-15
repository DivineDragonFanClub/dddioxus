use dioxus::prelude::*;

use crate::components::procs_view::{ProcTreeNode, Selection};
use crate::dev::{fixtures, StoryPage, StorySection};

#[component]
pub fn DevProcTreeNode() -> Element {
    let noop = use_callback(|_sel: Selection| {});
    let leaf = fixtures::sample_proc_node_leaf();
    let branch = fixtures::sample_proc_node_with_children();

    rsx! {
        StoryPage { title: "ProcTreeNode",
            StorySection { label: "Leaf node",
                div { class: "p-3 bg-gray-800 font-mono text-xs",
                    ProcTreeNode {
                        root_label: String::from("Root"),
                        path: vec![0],
                        node: leaf.clone(),
                        is_next: false,
                        selected: None,
                        on_select: noop,
                    }
                }
            }
            StorySection { label: "Branch with children (collapsed)",
                div { class: "p-3 bg-gray-800 font-mono text-xs",
                    ProcTreeNode {
                        root_label: String::from("Root"),
                        path: vec![1],
                        node: branch.clone(),
                        is_next: true,
                        selected: None,
                        on_select: noop,
                    }
                }
            }
            StorySection { label: "Selected",
                div { class: "p-3 bg-gray-800 font-mono text-xs",
                    ProcTreeNode {
                        root_label: String::from("Root"),
                        path: vec![0],
                        node: leaf,
                        is_next: false,
                        selected: Some(Selection {
                            root: "Root".into(),
                            path: vec![0],
                            name: "WaitForInput".into(),
                            hashcode: 0x1122_3344,
                        }),
                        on_select: noop,
                    }
                }
            }
        }
    }
}
