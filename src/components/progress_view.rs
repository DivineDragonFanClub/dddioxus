use dioxus::prelude::*;

use crate::components::chapter_view::ChapterView;
use crate::components::gmap_view::GmapView;
use crate::components::ui::{Page, TabBar};

// the world map (gmap nodes) and the chapter/story list are two views of the same "where are you in
// the game" idea, so they live under one Progress page as tabs. each tab mounts its own view, which
// loads lazily when shown (and re-fetches on switch, which is fine and keeps the data fresh)
#[component]
pub fn ProgressView() -> Element {
    let mut tab = use_signal(|| 0usize);

    rsx! {
        Page {
            div { class: "shrink-0 border-b border-gray-800",
                TabBar {
                    tabs: vec!["World Map".to_string(), "Chapters".to_string()],
                    selected: tab(),
                    on_select: move |i| tab.set(i),
                }
            }
            if tab() == 0 {
                GmapView {}
            } else {
                ChapterView {}
            }
        }
    }
}
