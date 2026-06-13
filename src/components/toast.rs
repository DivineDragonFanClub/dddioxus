use std::time::Duration;

use dioxus::prelude::*;

// how long a toast stays up before it removes itself
const TOAST_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, PartialEq)]
struct Toast {
    id: u64,
    message: String,
}

// handle to the global toast queue. grab it with use_toasts() anywhere under ToastProvider and
// call .show(msg) to pop a message
#[derive(Clone, Copy)]
pub struct Toasts {
    items: Signal<Vec<Toast>>,
    next_id: Signal<u64>,
}

impl Toasts {
    pub fn show(self, message: impl Into<String>) {
        let mut items = self.items;
        let mut next_id = self.next_id;
        let id = next_id();
        next_id.set(id.wrapping_add(1));
        items.write().push(Toast { id, message: message.into() });
        spawn(async move {
            tokio::time::sleep(TOAST_TTL).await;
            items.write().retain(|t| t.id != id);
        });
    }
}

pub fn use_toasts() -> Toasts {
    use_context::<Toasts>()
}

// wrap the app once near the root: hands the Toasts context to everything inside and floats the
// toast stack on top of it
#[component]
pub fn ToastProvider(children: Element) -> Element {
    let items = use_signal(Vec::<Toast>::new);
    let next_id = use_signal(|| 0u64);
    use_context_provider(|| Toasts { items, next_id });

    rsx! {
        {children}
        div { class: "fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none",
            for toast in items().into_iter() {
                div {
                    key: "{toast.id}",
                    class: "pointer-events-auto max-w-xs bg-gray-800/85 backdrop-blur-md border border-gray-700/70 text-gray-100 text-xs rounded-lg shadow-xl shadow-black/40 px-3 py-2.5",
                    "{toast.message}"
                }
            }
        }
    }
}
