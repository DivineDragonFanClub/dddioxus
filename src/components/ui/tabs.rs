use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct TabBarProps {
    pub tabs: Vec<String>,
    pub selected: usize,
    pub on_select: EventHandler<usize>,
    #[props(default = String::new())]
    pub class: String,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

// underline-style tab switcher. used wherever one page hosts a few sub-views
#[component]
pub fn TabBar(props: TabBarProps) -> Element {
    rsx! {
        div { class: "flex items-center gap-1 {props.class}", ..props.attributes,
            for (i, label) in props.tabs.iter().enumerate() {
                {
                    let active = i == props.selected;
                    let cls = if active {
                        "px-3 py-1.5 text-sm font-medium text-white border-b-2 border-indigo-500"
                    } else {
                        "px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 border-b-2 border-transparent"
                    };
                    rsx! {
                        button {
                            key: "{i}",
                            class: "{cls} cursor-pointer transition-colors",
                            onclick: move |_| props.on_select.call(i),
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
