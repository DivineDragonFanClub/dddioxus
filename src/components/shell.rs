use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::Route;

// the pages reachable from the top bar dropdown, in order. the dev storybook is appended on debug
// builds further down
fn pages() -> Vec<(&'static str, Route)> {
    vec![
        ("Scene", Route::Scene {}),
        ("Map", Route::Map {}),
        ("Progress", Route::Progress {}),
        ("Forces", Route::Forces {}),
        ("Bonds", Route::Bonds {}),
        ("Variables", Route::Variables {}),
        ("Procs", Route::Procs {}),
        ("Mess", Route::Mess {}),
        ("Cutscene", Route::Cutscene {}),
    ]
}

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
        if on_dev {
            // the storybook keeps its own sidebar, there are a lot of stories to scroll
            div { class: "flex flex-1 overflow-hidden min-h-0",
                {dev_sidebar()}
                div { class: "flex flex-col flex-1 overflow-hidden min-h-0",
                    Outlet::<Route> {}
                }
            }
        } else {
            div { class: "flex flex-col flex-1 overflow-hidden min-h-0",
                AppBar {}
                div { class: "flex flex-col flex-1 overflow-hidden min-h-0",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

// the top app bar: brand on the left, the page picker dropdown next to it, connection status and a
// disconnect button on the right
#[component]
fn AppBar() -> Element {
    let mut conn = use_context::<Signal<ConnectionState>>();

    let (dot, label, sub) = match &*conn.read() {
        ConnectionState::Connected { info, .. } => (
            "bg-emerald-500",
            format!("{}:{}", info.host, info.port),
            Some(format!("v{}", info.api_version)),
        ),
        ConnectionState::Reconnecting { .. } => ("bg-amber-400 animate-pulse", "Reconnecting".to_string(), None),
        ConnectionState::Disconnected { .. } => ("bg-gray-500", "Offline".to_string(), None),
    };
    let connected = matches!(&*conn.read(), ConnectionState::Connected { .. });

    rsx! {
        header { class: "relative z-50 flex items-center gap-3 px-4 h-12 shrink-0 bg-gray-900 border-b border-gray-800 shadow-lg shadow-black/20",
            div { class: "h-6 w-6 shrink-0 rounded-md bg-indigo-500 flex items-center justify-center text-xs font-bold text-white shadow-sm shadow-indigo-900/50 select-none",
                "D"
            }
            PageMenu {}
            div { class: "ml-auto flex items-center gap-3",
                div { class: "flex items-center gap-2",
                    span { class: "inline-flex h-2 w-2 rounded-full {dot}" }
                    span { class: "text-xs text-gray-300 font-mono", "{label}" }
                    if let Some(sub) = sub {
                        span { class: "text-xs text-gray-600 font-mono", "{sub}" }
                    }
                }
                if connected {
                    button {
                        class: "text-red-400 hover:text-red-300 text-xs cursor-pointer transition-colors",
                        onclick: move |_| {
                            let old = conn.peek().client().cloned();
                            conn.set(ConnectionState::Disconnected { reason: None });
                            if let Some(client) = old {
                                spawn(async move { client.close().await });
                            }
                        },
                        "Disconnect"
                    }
                }
            }
        }
    }
}

// the page picker. a button showing the current page, click to drop a translucent menu of every
// page. replaces the old left sidebar so pages get the full width
#[component]
fn PageMenu() -> Element {
    let nav = use_navigator();
    let current: Route = use_route();
    let mut open = use_signal(|| false);

    let mut items = pages();
    #[cfg(any(debug_assertions, feature = "dev"))]
    items.push(("UI Storybook", Route::DevIndex {}));

    let active = items
        .iter()
        .find(|(_, r)| std::mem::discriminant(r) == std::mem::discriminant(&current))
        .map(|(l, _)| *l)
        .unwrap_or("Menu");

    rsx! {
        div { class: "relative",
            button {
                class: "flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium text-white \
                        bg-gray-800/70 hover:bg-gray-700/70 border border-gray-700 cursor-pointer transition-colors",
                onclick: move |_| open.toggle(),
                span { "{active}" }
                span { class: "text-gray-500 text-xs", "\u{25BE}" }
            }
            if open() {
                // click-away catcher behind the menu
                div { class: "fixed inset-0 z-40", onclick: move |_| open.set(false) }
                div { class: "absolute left-0 mt-1 z-50 min-w-52 py-1 rounded-lg \
                              bg-gray-800/90 backdrop-blur-md border border-gray-700 shadow-2xl shadow-black/50",
                    for (label, route) in items.into_iter() {
                        {
                            let is_active = std::mem::discriminant(&route) == std::mem::discriminant(&current);
                            let cls = if is_active {
                                "flex items-center w-full text-left px-3 py-1.5 text-sm text-white bg-indigo-500/20"
                            } else {
                                "flex items-center w-full text-left px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700/60 hover:text-white cursor-pointer"
                            };
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: "{cls} transition-colors",
                                    onclick: move |_| {
                                        nav.push(route.clone());
                                        open.set(false);
                                    },
                                    if is_active {
                                        span { class: "mr-2 text-indigo-400", "\u{2022}" }
                                    } else {
                                        span { class: "mr-2 text-transparent", "\u{2022}" }
                                    }
                                    "{label}"
                                }
                            }
                        }
                    }
                }
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
