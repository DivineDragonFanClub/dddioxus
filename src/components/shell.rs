use dioxus::prelude::*;

use super::connection_provider::ConnectionProvider;
use crate::Route;

#[component]
pub fn Shell() -> Element {
    rsx! {
        ConnectionProvider {
            div { class: "flex flex-1 overflow-hidden",
                Sidebar {}
                div { class: "flex-1 overflow-hidden",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        nav { class: "w-40 shrink-0 bg-gray-950 border-r border-gray-700 flex flex-col py-2",
            NavItem { route: Route::Scene {}, label: "Scene" }
            NavItem { route: Route::Globals {}, label: "Globals" }
            NavItem { route: Route::Procs {}, label: "Procs" }
        }
    }
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
