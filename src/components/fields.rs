use dioxus::prelude::*;

use crate::protocol::Vec3;

#[derive(PartialEq, Clone, Props)]
pub struct Vec3EditorProps {
    pub label: &'static str,
    pub value: Vec3,
    pub on_change: EventHandler<Vec3>,
}

#[component]
pub fn Vec3Editor(props: Vec3EditorProps) -> Element {
    let mut x = use_signal(|| format!("{:.3}", props.value.x));
    let mut y = use_signal(|| format!("{:.3}", props.value.y));
    let mut z = use_signal(|| format!("{:.3}", props.value.z));

    let mut last_value = use_signal(|| props.value);
    if *last_value.read() != props.value {
        last_value.set(props.value);
        x.set(format!("{:.3}", props.value.x));
        y.set(format!("{:.3}", props.value.y));
        z.set(format!("{:.3}", props.value.z));
    }

    let submit = move |_| {
        let new_x = x().parse::<f32>().unwrap_or(props.value.x);
        let new_y = y().parse::<f32>().unwrap_or(props.value.y);
        let new_z = z().parse::<f32>().unwrap_or(props.value.z);
        props.on_change.call(Vec3 { x: new_x, y: new_y, z: new_z });
    };

    let input_class = "w-16 px-1 py-0.5 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-center";

    rsx! {
        div { class: "flex items-center gap-2 mb-1",
            span { class: "text-gray-400 w-16", "{props.label}" }
            span { class: "text-red-400", "X" }
            input {
                class: "{input_class}",
                value: "{x}",
                oninput: move |e| x.set(e.value()),
                onchange: submit,
            }
            span { class: "text-green-400", "Y" }
            input {
                class: "{input_class}",
                value: "{y}",
                oninput: move |e| y.set(e.value()),
                onchange: submit,
            }
            span { class: "text-blue-400", "Z" }
            input {
                class: "{input_class}",
                value: "{z}",
                oninput: move |e| z.set(e.value()),
                onchange: submit,
            }
        }
    }
}
