use dioxus::prelude::*;

use crate::protocol::StatValue;

#[derive(PartialEq, Clone, Props)]
pub struct StatFieldProps {
    pub stat: StatValue,
    pub on_commit: EventHandler<(i32, i32)>,
}

#[component]
pub fn StatField(props: StatFieldProps) -> Element {
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.stat.value.to_string());

    let commit = {
        let on_commit = props.on_commit;
        let stat_index = props.stat.index;
        let current = props.stat.value;
        move || {
            editing.set(false);
            if let Ok(v) = draft().trim().parse::<i32>() {
                if v != current {
                    on_commit.call((stat_index, v));
                }
            }
        }
    };

    rsx! {
        div { class: "flex flex-col items-center w-12",
            span { class: "text-gray-400 text-[10px] uppercase tracking-wide", "{props.stat.label}" }
            if editing() {
                input {
                    r#type: "text",
                    inputmode: "numeric",
                    class: "w-12 px-1.5 py-0.5 bg-gray-950 text-amber-300 rounded border border-gray-600 \
                            focus:border-indigo-500 focus:outline-none text-center text-sm tabular-nums",
                    value: "{draft}",
                    autofocus: true,
                    oninput: move |e| draft.set(e.value()),
                    onblur: {
                        let mut commit = commit.clone();
                        move |_| commit()
                    },
                    onkeydown: {
                        let mut commit = commit.clone();
                        move |e| {
                            if e.key() == Key::Enter { commit(); }
                            else if e.key() == Key::Escape { editing.set(false); }
                        }
                    },
                }
            } else {
                span {
                    class: "w-12 inline-block text-center text-amber-300 text-sm rounded cursor-text \
                            hover:bg-gray-700/50 tabular-nums",
                    onclick: {
                        let value = props.stat.value;
                        move |_| {
                            draft.set(value.to_string());
                            editing.set(true);
                        }
                    },
                    "{props.stat.value}"
                }
            }
        }
    }
}
