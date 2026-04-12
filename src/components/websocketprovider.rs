use dioxus::prelude::*;
use crate::hooks::websocket::do_connect;

#[derive(PartialEq, Clone, Props)]
pub struct WebsocketProviderProps {
    children: Element,
}

#[component]
pub fn WebSocketProvider(props: WebsocketProviderProps) -> Element {
    let mut url = use_signal(|| "ws://127.0.0.1:18050".to_string());
    let mut connecting = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

    let mut ws = crate::hooks::use_ws_provider();

    let connect = move |_| {
        if connecting() {
            return;
        }
        connecting.set(true);
        error_msg.set(None);
        spawn(async move {
            match do_connect(&url()).await {
                Ok(state) => {
                    let mut w = ws.write();
                    w.state = state;
                    w.is_open = true;
                }
                Err(err) => {
                    error_msg.set(Some(format!("Failed to connect: {err}")));
                }
            }
            connecting.set(false);
        });
    };

    rsx! {
        if ws().is_open() {
            { props.children }
        } else {
            div { class: "flex flex-col items-center gap-4 p-8",
                div { class: "flex gap-2 items-center",
                    input {
                        class: "border rounded px-3 py-2 text-sm font-mono w-64",
                        value: "{url}",
                        disabled: connecting(),
                        oninput: move |event| url.set(event.value()),
                    }
                    button {
                        class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg disabled:opacity-50",
                        disabled: connecting(),
                        onclick: connect,
                        if connecting() {
                            "Connecting..."
                        } else {
                            "Connect"
                        }
                    }
                }
                if let Some(err) = error_msg() {
                    p { class: "text-red-500 text-sm", "{err}" }
                }
            }
        }
    }
}