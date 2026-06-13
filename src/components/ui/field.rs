use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct SearchFieldProps {
    pub value: String,
    pub on_input: EventHandler<String>,
    #[props(into, default = "Filter\u{2026}".to_string())]
    pub placeholder: String,
    /// Width / layout classes (e.g. `w-56`), merged with the field's own.
    #[props(default = String::new())]
    pub class: String,
    /// Other forwarded attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

// the standard search/filter box. shows a clear button once you've typed something
#[component]
pub fn SearchField(props: SearchFieldProps) -> Element {
    let has_text = !props.value.is_empty();
    rsx! {
        div {
            class: "relative {props.class}",
            ..props.attributes,
            input {
                class: "w-full pl-3 pr-8 py-1.5 bg-gray-900/60 text-white rounded-md border border-gray-700 \
                        focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500/40 \
                        text-sm placeholder:text-gray-500",
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                oninput: move |e| props.on_input.call(e.value()),
            }
            if has_text {
                button {
                    class: "absolute inset-y-0 right-2 flex items-center text-gray-500 hover:text-white text-sm cursor-pointer",
                    onclick: move |_| props.on_input.call(String::new()),
                    "\u{2715}"
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct EditableNumberProps {
    pub value: i64,
    pub on_commit: EventHandler<i64>,
    #[props(default = "w-14".to_string())]
    pub width: String,
    #[props(default = false)]
    pub disabled: bool,
}

// click the number to edit it, Enter or click-away commits, Escape cancels. the shared inline
// editor for stats, levels, item uses, bond levels, anything that's a plain integer
#[component]
pub fn EditableNumber(props: EditableNumberProps) -> Element {
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.value.to_string());
    let value = props.value;
    let on_commit = props.on_commit;
    let width = props.width.clone();

    let commit = use_callback(move |()| {
        editing.set(false);
        if let Ok(n) = draft().trim().parse::<i64>() {
            if n != value {
                on_commit.call(n);
            }
        }
    });

    if editing() {
        rsx! {
            input {
                // text + numeric inputmode instead of type=number, so there are no spinner arrows
                // stealing width and clipping the value
                r#type: "text",
                inputmode: "numeric",
                class: "{width} px-1.5 py-0.5 bg-gray-950 text-amber-300 rounded border border-gray-600 \
                        focus:border-indigo-500 focus:outline-none text-center text-sm tabular-nums",
                value: "{draft}",
                autofocus: true,
                oninput: move |e| draft.set(e.value()),
                onblur: move |_| commit.call(()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        commit.call(());
                    } else if e.key() == Key::Escape {
                        editing.set(false);
                    }
                },
            }
        }
    } else {
        rsx! {
            span {
                class: "{width} inline-block px-1.5 py-0.5 text-amber-300 text-center text-sm rounded \
                        border border-dashed border-amber-400/30 hover:border-amber-400/70 hover:bg-gray-700/40 \
                        cursor-text tabular-nums transition-colors",
                title: "Click to edit",
                onclick: move |_| {
                    if !props.disabled {
                        draft.set(value.to_string());
                        editing.set(true);
                    }
                },
                "{value}"
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct EditableTextProps {
    pub value: String,
    pub on_commit: EventHandler<String>,
    #[props(default = "w-40".to_string())]
    pub width: String,
    #[props(default = false)]
    pub disabled: bool,
}

// same click-to-edit feel as EditableNumber but for free text (string variables, names)
#[component]
pub fn EditableText(props: EditableTextProps) -> Element {
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.value.clone());
    let on_commit = props.on_commit;
    let width = props.width.clone();
    let current = props.value.clone();

    let commit = {
        let current = current.clone();
        use_callback(move |()| {
            editing.set(false);
            let d = draft();
            if d != current {
                on_commit.call(d);
            }
        })
    };

    if editing() {
        rsx! {
            input {
                r#type: "text",
                class: "{width} px-2 py-0.5 bg-gray-950 text-amber-300 rounded border border-gray-600 \
                        focus:border-indigo-500 focus:outline-none text-sm",
                value: "{draft}",
                autofocus: true,
                oninput: move |e| draft.set(e.value()),
                onblur: move |_| commit.call(()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        commit.call(());
                    } else if e.key() == Key::Escape {
                        editing.set(false);
                    }
                },
            }
        }
    } else {
        let display = props.value.clone();
        rsx! {
            span {
                class: "{width} inline-block px-2 py-0.5 text-amber-300 text-sm truncate rounded \
                        border border-dashed border-amber-400/30 hover:border-amber-400/70 hover:bg-gray-700/40 \
                        cursor-text transition-colors",
                title: "Click to edit",
                onclick: move |_| {
                    if !props.disabled {
                        draft.set(display.clone());
                        editing.set(true);
                    }
                },
                "{props.value}"
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct CheckboxProps {
    pub checked: bool,
    pub on_toggle: EventHandler<bool>,
    #[props(default)]
    pub label: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = String::new())]
    pub class: String,
    /// Other forwarded attributes (`title` tooltip, `data-*`).
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

// native checkbox tinted with the app accent, with an optional inline label
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    rsx! {
        label {
            class: "inline-flex items-center gap-1.5 cursor-pointer select-none {props.class}",
            ..props.attributes,
            input {
                r#type: "checkbox",
                class: "accent-indigo-500 cursor-pointer",
                checked: props.checked,
                disabled: props.disabled,
                onchange: move |e| props.on_toggle.call(e.checked()),
            }
            if let Some(l) = props.label.as_ref() {
                span { class: "text-xs text-gray-300", "{l}" }
            }
        }
    }
}
