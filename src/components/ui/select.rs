use dioxus::prelude::*;

/// `Neutral` is a plain picker, `Action` tints it emerald for "add / assign" dropdowns.
#[derive(PartialEq, Clone, Copy, Default)]
pub enum SelectTone {
    #[default]
    Neutral,
    Action,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum SelectSize {
    #[default]
    Md,
    Sm,
}

#[derive(PartialEq, Clone, Props)]
pub struct SelectProps {
    pub on_change: EventHandler<String>,
    #[props(default)]
    pub tone: SelectTone,
    #[props(default)]
    pub size: SelectSize,
    /// Layout classes (`w-full`, `shrink-0`), merged with the select's own.
    #[props(default = String::new())]
    pub class: String,
    /// Other forwarded attributes (`title`, `disabled`, `data-*`).
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Plain `option { value, selected, "label" }` children; mark the current one `selected`.
    pub children: Element,
}

/// The one dropdown style. Callers supply the option children and mark the current one selected,
/// same as a normal html select.
///
/// ```rust,ignore
/// Select { value: state, on_change: move |v| set(v),
///     option { value: "0", selected: state == "0", "Hidden" }
///     option { value: "1", selected: state == "1", "Unlocked" }
/// }
/// ```
#[component]
pub fn Select(props: SelectProps) -> Element {
    let tone = match props.tone {
        SelectTone::Neutral => "text-gray-200",
        SelectTone::Action => "text-emerald-300",
    };
    let size = match props.size {
        SelectSize::Md => "text-sm px-2 py-1",
        SelectSize::Sm => "text-xs px-1.5 py-0.5",
    };
    rsx! {
        select {
            class: "bg-gray-900/70 {tone} {size} rounded-md border border-gray-700 \
                    focus:border-indigo-500 focus:outline-none cursor-pointer {props.class}",
            onchange: move |e| props.on_change.call(e.value()),
            ..props.attributes,
            {props.children}
        }
    }
}
