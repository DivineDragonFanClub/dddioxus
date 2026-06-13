use dioxus::prelude::*;

use crate::components::chapter_view::ChapterView;
use crate::components::gmap_view::GmapView;

// the world map (gmap nodes) and the chapter/story list are two views of the same "where are you in
// the game" idea, so they live under one Progress page as tabs. each tab mounts its own view, which
// loads lazily when shown (and re-fetches on switch, which is fine and keeps the data fresh)
#[component]
pub fn ProgressView() -> Element {
    let mut tab = use_signal(|| 0usize);

    let tab_class = move |idx: usize| {
        if tab() == idx {
            "px-3 py-1.5 text-sm text-white border-b-2 border-indigo-500"
        } else {
            "px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 border-b-2 border-transparent"
        }
    };

    rsx! {
        div { class: "flex flex-col flex-1 min-h-0",
            div { class: "flex items-center gap-1 px-4 bg-gray-900 border-b border-gray-700 shrink-0",
                button { class: tab_class(0), onclick: move |_| tab.set(0), "World Map" }
                button { class: tab_class(1), onclick: move |_| tab.set(1), "Chapters" }
            }
            div { class: "flex flex-col flex-1 min-h-0",
                if tab() == 0 {
                    GmapView {}
                } else {
                    ChapterView {}
                }
            }
        }
    }
}
