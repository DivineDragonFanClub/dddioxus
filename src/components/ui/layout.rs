use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct PageProps {
    /// True lays the children out in a row (list + inspector), false stacks them.
    #[props(default = false)]
    pub row: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// The padded region a page's cards live in. Sits on the app's dark background with a gap between
/// cards so the elevated surfaces read as separate panels.
#[component]
pub fn Page(props: PageProps) -> Element {
    let dir = if props.row { "flex-row" } else { "flex-col" };
    rsx! {
        div { class: "flex {dir} flex-1 min-h-0 gap-3 p-3 {props.class}", ..props.attributes, {props.children} }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ToolbarProps {
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A horizontal row of controls (search, filters, buttons). Transparent, meant to sit above cards
/// or inside a card header.
#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    rsx! {
        div { class: "flex items-center gap-2 shrink-0 {props.class}", ..props.attributes, {props.children} }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum StateKind {
    Loading,
    Error,
    Empty,
}

#[derive(PartialEq, Clone, Props)]
pub struct EmptyStateProps {
    pub kind: StateKind,
    #[props(into)]
    pub message: String,
    /// Compact drops the big padding for inline use inside a tight list.
    #[props(default = false)]
    pub compact: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The single source of truth for loading / error / nothing-here messages so they stop drifting
/// between gray-400, gray-500, and red across pages.
#[component]
pub fn EmptyState(props: EmptyStateProps) -> Element {
    let pad = if props.compact { "py-1" } else { "p-6" };
    let look = match props.kind {
        StateKind::Loading => "text-gray-400",
        StateKind::Error => "text-red-400",
        StateKind::Empty => "text-gray-500 italic",
    };
    rsx! {
        p { class: "{look} text-sm {pad} {props.class}", ..props.attributes, "{props.message}" }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ListRowProps {
    #[props(default = false)]
    pub selected: bool,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(default = String::new())]
    pub class: String,
    /// Other forwarded attributes: `key`, `title`, `data-*`, and so on.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// One row in a scrollable list. Handles the hover wash and the selected highlight so every list
/// feels the same.
#[component]
pub fn ListRow(props: ListRowProps) -> Element {
    let state = if props.selected {
        "bg-indigo-500/15 ring-1 ring-indigo-500/40"
    } else {
        "hover:bg-gray-700/50"
    };
    let clickable = if props.onclick.is_some() { "cursor-pointer" } else { "" };
    rsx! {
        div {
            class: "flex items-center gap-3 px-2 py-1.5 rounded-md transition-colors {state} {clickable} {props.class}",
            onclick: move |e| {
                if let Some(h) = props.onclick.as_ref() {
                    h.call(e);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct FieldProps {
    #[props(into)]
    pub label: String,
    /// Width of the label column, tweak when labels are long or the panel is narrow.
    #[props(into, default = "w-20".to_string())]
    pub label_width: String,
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A labeled field row: muted label on the left, the control on the right. Stacks cleanly in narrow
/// inspectors so dropdowns and editors never collide on one line.
///
/// ```rust,ignore
/// Field { label: "Class", Select { class: "w-full", on_change, /* options */ } }
/// ```
#[component]
pub fn Field(props: FieldProps) -> Element {
    rsx! {
        div { class: "flex items-center gap-2 min-w-0 {props.class}", ..props.attributes,
            span { class: "{props.label_width} shrink-0 text-gray-400 text-xs", "{props.label}" }
            div { class: "flex-1 min-w-0 flex items-center gap-2", {props.children} }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct SectionLabelProps {
    #[props(into)]
    pub label: String,
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The small all-caps label that introduces a group of rows.
#[component]
pub fn SectionLabel(props: SectionLabelProps) -> Element {
    rsx! {
        p { class: "text-xs uppercase tracking-wide text-gray-500 {props.class}", ..props.attributes, "{props.label}" }
    }
}
