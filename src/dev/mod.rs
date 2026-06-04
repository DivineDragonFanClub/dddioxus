pub mod fixtures;
pub mod stories;

use dioxus::prelude::*;

use crate::Route;

pub use stories::*;

pub fn is_dev_route(route: &Route) -> bool {
    matches!(
        route,
        Route::DevIndex {}
            | Route::DevVec3Editor {}
            | Route::DevSceneTree {}
            | Route::DevScenePanel {}
            | Route::DevGlobalRow {}
            | Route::DevGlobalsPanel {}
            | Route::DevComponentRow {}
            | Route::DevComponentsListPanel {}
            | Route::DevTransformPanel {}
            | Route::DevProcTreeNode {}
            | Route::DevDescsPanel {}
            | Route::DevProcsPanel {}
            | Route::DevSceneSimulation {}
    )
}

#[component]
pub fn DevSidebar() -> Element {
    rsx! {
        nav { class: "w-52 shrink-0 bg-gray-900 border-r border-gray-700 flex flex-col py-2 overflow-auto",
            Link {
                to: Route::Scene {},
                class: "block px-4 py-2 text-xs text-gray-400 hover:text-indigo-400 border-b border-gray-800 mb-2",
                "← Back to app"
            }
            SidebarHeader { label: "Stories" }
            DevNavItem { route: Route::DevIndex {}, label: "Index" }
            SidebarHeader { label: "Leaves" }
            DevNavItem { route: Route::DevVec3Editor {}, label: "Vec3Editor" }
            DevNavItem { route: Route::DevGlobalRow {}, label: "GlobalRow" }
            DevNavItem { route: Route::DevComponentRow {}, label: "ComponentRow" }
            DevNavItem { route: Route::DevProcTreeNode {}, label: "ProcTreeNode" }
            SidebarHeader { label: "Panels" }
            DevNavItem { route: Route::DevScenePanel {}, label: "ScenePanel" }
            DevNavItem { route: Route::DevGlobalsPanel {}, label: "GlobalsPanel" }
            DevNavItem { route: Route::DevComponentsListPanel {}, label: "ComponentsListPanel" }
            DevNavItem { route: Route::DevTransformPanel {}, label: "TransformPanel" }
            DevNavItem { route: Route::DevDescsPanel {}, label: "DescsPanel" }
            DevNavItem { route: Route::DevProcsPanel {}, label: "ProcsPanel" }
            SidebarHeader { label: "Widgets" }
            DevNavItem { route: Route::DevSceneTree {}, label: "SceneTree" }
            SidebarHeader { label: "Simulations" }
            DevNavItem { route: Route::DevSceneSimulation {}, label: "Scene viewer" }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct SidebarHeaderProps {
    label: &'static str,
}

#[component]
fn SidebarHeader(props: SidebarHeaderProps) -> Element {
    rsx! {
        p { class: "px-3 pt-3 pb-1 text-[10px] uppercase tracking-wide text-gray-500", "{props.label}" }
    }
}

#[derive(PartialEq, Clone, Props)]
struct DevNavItemProps {
    route: Route,
    label: &'static str,
}

#[component]
fn DevNavItem(props: DevNavItemProps) -> Element {
    let current: Route = use_route();
    let active = std::mem::discriminant(&current) == std::mem::discriminant(&props.route);
    let class = if active {
        "block px-4 py-1.5 text-xs text-white bg-indigo-600"
    } else {
        "block px-4 py-1.5 text-xs text-gray-300 hover:bg-gray-800"
    };
    rsx! {
        Link { to: props.route.clone(), class: "{class}", "{props.label}" }
    }
}

#[component]
pub fn DevIndex() -> Element {
    rsx! {
        div { class: "p-8 text-gray-200 font-mono",
            h1 { class: "text-2xl font-bold text-white mb-4", "Component Stories" }
            p { class: "text-gray-400 mb-6 max-w-2xl text-sm",
                "Storybook-style previews of every UI component, mounted with canned fixtures. Use the left sidebar to jump between stories. No live game connection required."
            }
            p { class: "text-gray-500 text-xs",
                "This page is gated behind "
                code { class: "text-yellow-300", "cfg(any(debug_assertions, feature = \"dev\"))" }
                " and does not ship in release builds without the "
                code { class: "text-yellow-300", "dev" }
                " feature enabled."
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct StoryPageProps {
    pub title: &'static str,
    pub children: Element,
}

#[component]
pub fn StoryPage(props: StoryPageProps) -> Element {
    rsx! {
        div { class: "h-full overflow-y-auto",
            div { class: "p-6 space-y-6",
                h1 { class: "text-xl font-bold text-white border-b border-gray-700 pb-2", "{props.title}" }
                {props.children}
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct StorySectionProps {
    pub label: &'static str,
    pub children: Element,
}

#[component]
pub fn StorySection(props: StorySectionProps) -> Element {
    rsx! {
        div { class: "space-y-2",
            p { class: "text-[10px] uppercase tracking-wide text-gray-500", "{props.label}" }
            div { class: "bg-gray-900 border border-gray-800 rounded overflow-hidden",
                {props.children}
            }
        }
    }
}
