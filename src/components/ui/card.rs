use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct CardProps {
    /// A simple title shows the standard header strip. Omit for a bare surface. Accepts dynamic
    /// strings (`title: format!("{n} units")`) as well as literals.
    #[props(into)]
    pub title: Option<String>,
    /// Right-aligned header content (refresh button, filters, counts). Showing a header also forces
    /// the header strip even without a title.
    #[props(default)]
    pub header: Option<Element>,
    /// Set false to drop the default body padding (lists/trees manage their own).
    #[props(default = true)]
    pub padded: bool,
    /// Width/flex classes for placing the card (`flex-1`, `w-56 shrink-0`), merged with the card's
    /// own surface classes.
    #[props(default = String::new())]
    pub class: String,
    /// Extra classes for the scrolling body.
    #[props(default = String::new())]
    pub body_class: String,
    /// Any other forwarded attribute (`style`, `id`, `title`, `data-*`, events).
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// The elevated surface everything sits on: dark translucent fill, soft border, drop shadow, rounded
/// corners. Give it a `title` and/or a `header` slot for the header strip, or leave it bare.
///
/// ```rust,ignore
/// Card { title: "World Map", header: rsx! { Button { onclick, "Refresh" } },
///     // body
/// }
/// Card { class: "flex-1", padded: false, /* a scrolling list */ }
/// ```
#[component]
pub fn Card(props: CardProps) -> Element {
    let title = props.title.filter(|t| !t.is_empty());
    let has_header = title.is_some() || props.header.is_some();
    let body_pad = if props.padded { "p-3" } else { "" };
    rsx! {
        div {
            class: "flex flex-col min-h-0 bg-gray-800/80 border border-gray-700/70 rounded-xl \
                    shadow-lg shadow-black/30 overflow-hidden {props.class}",
            ..props.attributes,
            if has_header {
                div { class: "flex items-center gap-2 px-3 py-2 border-b border-gray-700/70 bg-gray-900/40 shrink-0",
                    if let Some(t) = title {
                        h3 { class: "text-white font-semibold text-sm truncate", "{t}" }
                    }
                    div { class: "ml-auto flex items-center gap-2", {props.header} }
                }
            }
            div { class: "flex-1 overflow-auto {body_pad} {props.body_class}", {props.children} }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct PanelHeaderProps {
    #[props(into)]
    pub title: String,
    #[props(into, default)]
    pub subtitle: Option<String>,
    #[props(default)]
    pub actions: Option<Element>,
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Header strip for a sub-panel inside a card (an inspector section, a sidebar list title).
#[component]
pub fn PanelHeader(props: PanelHeaderProps) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2 px-3 py-2 border-b border-gray-700/70 bg-gray-900/40 shrink-0 {props.class}",
            ..props.attributes,
            div { class: "min-w-0",
                h3 { class: "text-white font-semibold text-sm truncate", "{props.title}" }
                if let Some(s) = props.subtitle.as_ref() {
                    p { class: "text-gray-500 text-xs truncate", "{s}" }
                }
            }
            div { class: "ml-auto flex items-center gap-2 shrink-0", {props.actions} }
        }
    }
}
