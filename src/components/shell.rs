use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Shell() -> Element {
    let current: Route = use_route();

    #[cfg(any(debug_assertions, feature = "dev"))]
    let on_dev = crate::dev::is_dev_route(&current);
    #[cfg(not(any(debug_assertions, feature = "dev")))]
    let on_dev = false;
    #[cfg(not(any(debug_assertions, feature = "dev")))]
    let _ = &current;

    rsx! {
        div {
            "data-component": "Shell",
            class: "flex flex-1 overflow-hidden min-h-0",
            if on_dev {
                {dev_sidebar()}
            } else {
                Sidebar {}
            }
            div { class: "flex flex-col flex-1 overflow-hidden min-h-0",
                Outlet::<Route> {}
            }
        }
    }
}

#[cfg(any(debug_assertions, feature = "dev"))]
fn dev_sidebar() -> Element {
    rsx! { crate::dev::DevSidebar {} }
}

#[cfg(not(any(debug_assertions, feature = "dev")))]
fn dev_sidebar() -> Element {
    rsx! {}
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        nav { class: "w-40 shrink-0 bg-gray-950 border-r border-gray-700 flex flex-col py-2",
            NavItem { route: Route::Scene {}, label: "Scene" }
            NavItem { route: Route::Map {}, label: "Map" }
            NavItem { route: Route::Progress {}, label: "Progress" }
            NavItem { route: Route::Forces {}, label: "Forces" }
            NavItem { route: Route::Bonds {}, label: "Bonds" }
            NavItem { route: Route::Variables {}, label: "Variables" }
            NavItem { route: Route::Procs {}, label: "Procs" }
            NavItem { route: Route::Mess {}, label: "Mess" }
            NavItem { route: Route::Cutscene {}, label: "Cutscene" }
            {dev_nav_item()}
        }
    }
}

#[cfg(any(debug_assertions, feature = "dev"))]
fn dev_nav_item() -> Element {
    rsx! {
        div { class: "mt-auto pt-2 border-t border-gray-800",
            NavItem { route: Route::DevIndex {}, label: "UI Storybook" }
        }
    }
}

#[cfg(not(any(debug_assertions, feature = "dev")))]
fn dev_nav_item() -> Element {
    rsx! {}
}

#[derive(PartialEq, Clone, Props)]
struct NavItemProps {
    route: Route,
    label: &'static str,
}

#[component]
fn NavItem(props: NavItemProps) -> Element {
    let current: Route = use_route();
    let active = std::mem::discriminant(&current) == std::mem::discriminant(&props.route);
    let class = if active {
        "block px-4 py-2 text-sm text-white bg-indigo-600"
    } else {
        "block px-4 py-2 text-sm text-gray-300 hover:bg-gray-800"
    };

    rsx! {
        Link { to: props.route.clone(), class: "{class}", "{props.label}" }
    }
}
