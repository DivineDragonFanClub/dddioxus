use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    ClassInfo, GetClassesRequest, GetItemsRequest, GetSkillsRequest, ItemCatalogEntry,
    SkillCatalogEntry,
};
use crate::rpc;

#[derive(Clone, Default, PartialEq)]
pub struct Catalogs {
    pub classes: Vec<ClassInfo>,
    pub items: Vec<ItemCatalogEntry>,
    pub skills: Vec<SkillCatalogEntry>,
}

#[derive(PartialEq, Clone, Props)]
pub struct CatalogProviderProps {
    pub children: Element,
}

#[component]
pub fn CatalogProvider(props: CatalogProviderProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut catalogs = use_signal(Catalogs::default);
    use_context_provider(|| catalogs);

    let mut mounted = use_signal(|| false);
    if !mounted() {
        mounted.set(true);
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetClassesRequest).await {
                catalogs.with_mut(|c| c.classes = resp.classes);
            }
        });
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetItemsRequest).await {
                catalogs.with_mut(|c| c.items = resp.items);
            }
        });
        spawn(async move {
            if let Ok(resp) = rpc::call(&conn, GetSkillsRequest).await {
                catalogs.with_mut(|c| c.skills = resp.skills);
            }
        });
    }

    rsx! { {props.children} }
}
