use dioxus::prelude::*;

/// The accent color of a control. `Indigo` is the default action color, `Emerald` reads as
/// "add / confirm", `Red` is destructive, `Gray` is a neutral secondary.
#[derive(PartialEq, Clone, Copy, Default)]
pub enum Tone {
    #[default]
    Indigo,
    Emerald,
    Red,
    Gray,
}

/// How filled-in the button looks. `Solid` is a full colored block, `Outline` is a bordered pill,
/// `Ghost` is just colored text that lights up on hover (icon buttons, weak inline actions).
#[derive(PartialEq, Clone, Copy, Default)]
pub enum ButtonVariant {
    #[default]
    Solid,
    Outline,
    Ghost,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum ButtonSize {
    #[default]
    Md,
    Sm,
}

fn tone_classes(tone: Tone, variant: ButtonVariant) -> &'static str {
    use ButtonVariant::*;
    use Tone::*;
    match (variant, tone) {
        (Solid, Indigo) => "bg-indigo-500 hover:bg-indigo-600 text-white shadow-sm shadow-indigo-900/40",
        (Solid, Emerald) => "bg-emerald-600 hover:bg-emerald-500 text-white shadow-sm shadow-emerald-900/40",
        (Solid, Red) => "bg-red-600 hover:bg-red-500 text-white shadow-sm shadow-red-900/40",
        (Solid, Gray) => "bg-gray-700 hover:bg-gray-600 text-white shadow-sm",
        (Outline, Indigo) => "border border-indigo-500/40 text-indigo-300 hover:text-indigo-200 hover:border-indigo-500/70 hover:bg-indigo-500/10",
        (Outline, Emerald) => "border border-emerald-500/40 text-emerald-300 hover:text-emerald-200 hover:border-emerald-500/70 hover:bg-emerald-500/10",
        (Outline, Red) => "border border-red-500/40 text-red-400 hover:text-red-300 hover:border-red-500/70 hover:bg-red-500/10",
        (Outline, Gray) => "border border-gray-600 text-gray-300 hover:text-white hover:border-gray-500 hover:bg-gray-700/40",
        (Ghost, Indigo) => "text-indigo-300 hover:text-indigo-200 hover:bg-gray-700/50",
        (Ghost, Emerald) => "text-emerald-300 hover:text-emerald-200 hover:bg-gray-700/50",
        (Ghost, Red) => "text-red-400 hover:text-red-300 hover:bg-gray-700/50",
        (Ghost, Gray) => "text-gray-400 hover:text-white hover:bg-gray-700/50",
    }
}

fn size_classes(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Md => "px-4 py-1.5 text-sm rounded-md",
        ButtonSize::Sm => "px-2 py-1 text-xs rounded-md",
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ButtonProps {
    /// Accent color. Defaults to `Indigo`.
    #[props(default)]
    pub tone: Tone,
    /// Fill style: `Solid` (default), `Outline`, or `Ghost`.
    #[props(default)]
    pub variant: ButtonVariant,
    /// `Md` (default) or `Sm`.
    #[props(default)]
    pub size: ButtonSize,
    #[props(default = false)]
    pub disabled: bool,
    /// Shows a spinner and blocks clicks while true.
    #[props(default = false)]
    pub loading: bool,
    /// Stretch to fill the parent width.
    #[props(default = false)]
    pub full_width: bool,
    /// Optional element rendered before the label (an icon, a dot).
    #[props(default)]
    pub leading: Option<Element>,
    /// Optional element rendered after the label.
    #[props(default)]
    pub trailing: Option<Element>,
    /// Layout classes merged with the button's own styling (e.g. `ml-auto`, `flex-1`).
    #[props(default = String::new())]
    pub class: String,
    pub onclick: EventHandler<MouseEvent>,
    /// Any extra html attribute or global event: `class` (merged with the button's own classes),
    /// `style`, `id`, `title`, `data-*`, `aria-*`, `onmouseenter`, and so on.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// The one button in the app. Pick a `tone` + `variant` + `size`, everything else (focus ring,
/// transitions, disabled dimming, spinner) comes for free so every button matches. Forwards any
/// extra attribute, so `Button { class: "ml-auto", title: "Save", "data-testid": "save", onclick, "Save" }`
/// all work without the component declaring those props.
///
/// ```rust,ignore
/// Button { onclick: move |_| save(), "Save" }
/// Button { tone: Tone::Red, variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick, "Delete" }
/// Button { loading: saving(), full_width: true, onclick, "Submit" }
/// ```
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base = "focus:outline-none transition-colors inline-flex items-center justify-center gap-1.5 font-medium leading-none select-none";
    let tone = tone_classes(props.tone, props.variant);
    let size = size_classes(props.size);
    let disabled = props.disabled || props.loading;
    let state = if disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" };
    let w = if props.full_width { "w-full" } else { "" };
    let onclick = props.onclick;

    rsx! {
        button {
            class: "{base} {tone} {size} {state} {w} {props.class}",
            disabled: disabled,
            onclick: move |e| {
                if !disabled {
                    onclick.call(e);
                }
            },
            ..props.attributes,
            if props.loading {
                span { class: "inline-block h-3 w-3 rounded-full border-2 border-current border-t-transparent animate-spin" }
            }
            if let Some(leading) = props.leading {
                {leading}
            }
            {props.children}
            if let Some(trailing) = props.trailing {
                {trailing}
            }
        }
    }
}
