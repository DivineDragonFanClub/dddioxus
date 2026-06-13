use dioxus::prelude::*;

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Card, Checkbox, EditableNumber, EditableText, EmptyState,
    Field, ListRow, PanelHeader, SearchField, SectionLabel, Select, SelectSize, SelectTone,
    StateKind, TabBar, Tone,
};
use crate::dev::{StoryPage, StorySection};

// a live gallery of the shared UI kit. every primitive the rest of the app composes, shown in one
// place so you can eyeball the look and the variants without a game connection
#[component]
pub fn DevUiKit() -> Element {
    let mut search = use_signal(String::new);
    let mut number = use_signal(|| 42i64);
    let mut text = use_signal(|| "Alear".to_string());
    let mut checked = use_signal(|| true);
    let mut tab = use_signal(|| 0usize);

    rsx! {
        StoryPage { title: "UI Kit",
            StorySection { label: "Buttons \u{00B7} tones (solid)",
                div { class: "flex flex-wrap items-center gap-2 p-3",
                    Button { onclick: move |_| {}, "Primary" }
                    Button { tone: Tone::Emerald, onclick: move |_| {}, "Confirm" }
                    Button { tone: Tone::Red, onclick: move |_| {}, "Delete" }
                    Button { tone: Tone::Gray, onclick: move |_| {}, "Neutral" }
                    Button { disabled: true, onclick: move |_| {}, "Disabled" }
                }
            }
            StorySection { label: "Buttons \u{00B7} outline + ghost + small",
                div { class: "flex flex-wrap items-center gap-2 p-3",
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| {}, "Set current" }
                    Button { tone: Tone::Emerald, variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| {}, "Spawn" }
                    Button { tone: Tone::Red, variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| {}, "Despawn" }
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| {}, "Ghost" }
                    Button { tone: Tone::Red, variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| {}, "\u{2715}" }
                }
            }
            StorySection { label: "Buttons \u{00B7} loading, icon slot, full width",
                div { class: "flex flex-col gap-2 p-3 max-w-xs",
                    div { class: "flex items-center gap-2",
                        Button { loading: true, onclick: move |_| {}, "Saving" }
                        Button {
                            tone: Tone::Emerald,
                            leading: rsx! { span { class: "inline-block h-1.5 w-1.5 rounded-full bg-current" } },
                            onclick: move |_| {},
                            "With icon"
                        }
                    }
                    Button { full_width: true, onclick: move |_| {}, "Full width" }
                }
            }
            StorySection { label: "Fields (labeled rows, stack cleanly when narrow)",
                div { class: "p-3 space-y-1.5 max-w-sm",
                    Field { label: "Level",
                        EditableNumber { value: number(), on_commit: move |v| number.set(v) }
                    }
                    Field { label: "Class",
                        Select { size: SelectSize::Sm, class: "w-full", on_change: move |_| {},
                            option { value: "0", "Lord" }
                            option { value: "1", selected: true, "Paladin" }
                            option { value: "2", "Hero" }
                        }
                    }
                }
            }
            StorySection { label: "Inputs",
                div { class: "flex flex-wrap items-center gap-4 p-3",
                    SearchField { value: search(), class: "w-64", on_input: move |v| search.set(v) }
                    div { class: "flex items-center gap-2",
                        span { class: "text-gray-400 text-xs", "Number" }
                        EditableNumber { value: number(), on_commit: move |v| number.set(v) }
                    }
                    div { class: "flex items-center gap-2",
                        span { class: "text-gray-400 text-xs", "Text" }
                        EditableText { value: text(), on_commit: move |v| text.set(v) }
                    }
                    Checkbox { checked: checked(), label: "Acted", on_toggle: move |v| checked.set(v) }
                }
            }
            StorySection { label: "Selects",
                div { class: "flex flex-wrap items-center gap-3 p-3",
                    Select { size: SelectSize::Sm, on_change: move |_| {},
                        option { value: "0", "Hidden" }
                        option { value: "1", selected: true, "Unlocked" }
                        option { value: "2", "Locked" }
                    }
                    Select { tone: SelectTone::Action, size: SelectSize::Sm, on_change: move |_| {},
                        option { value: "", selected: true, "Add item\u{2026}" }
                        option { value: "1", "Iron Sword" }
                        option { value: "2", "Steel Lance" }
                    }
                }
            }
            StorySection { label: "Tabs",
                div { class: "p-3",
                    TabBar {
                        tabs: vec!["World Map".to_string(), "Chapters".to_string(), "Units".to_string()],
                        selected: tab(),
                        on_select: move |i| tab.set(i),
                    }
                }
            }
            StorySection { label: "List rows",
                div { class: "p-3",
                    SectionLabel { label: "3 nodes", class: "mb-2" }
                    ListRow {
                        span { class: "flex-1 text-white", "Firene Castle" }
                        span { class: "text-gray-500 text-xs font-mono", "CID_M005" }
                    }
                    ListRow {
                        selected: true,
                        span { class: "flex-1 text-white", "Brodia Keep" }
                        span { class: "text-amber-300 text-xs", "selected" }
                    }
                    ListRow {
                        span { class: "flex-1 text-white", "Solm Palace" }
                        span { class: "text-gray-500 text-xs font-mono", "CID_M017" }
                    }
                }
            }
            StorySection { label: "Card + panel header",
                div { class: "p-3 h-64",
                    Card {
                        title: "Units",
                        header: rsx! {
                            Button { size: ButtonSize::Sm, onclick: move |_| {}, "Refresh" }
                        },
                        PanelHeader { title: "Inspector", subtitle: "Alear / Lord" }
                        div { class: "p-3",
                            p { class: "text-gray-300 text-sm", "Card body content sits on the elevated surface." }
                        }
                    }
                }
            }
            StorySection { label: "Empty states",
                div { class: "p-3 space-y-1",
                    EmptyState { kind: StateKind::Loading, message: "Loading\u{2026}" }
                    EmptyState { kind: StateKind::Error, message: "Error: connection lost" }
                    EmptyState { kind: StateKind::Empty, message: "Nothing here yet" }
                }
            }
        }
    }
}
