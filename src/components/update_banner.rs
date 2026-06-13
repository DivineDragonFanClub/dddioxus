use dioxus::prelude::*;

// the /releases list (newest first) instead of /releases/latest, so prereleases count too. drafts
// aren't returned to unauthenticated callers, but we skip them defensively anyway
const REPO_API: &str = "https://api.github.com/repos/DivineDragonFanClub/dddioxus/releases?per_page=10";

#[derive(Clone, PartialEq)]
struct UpdateInfo {
    version: String,
    url: String,
}

// leading digits of a version component, so "2", "2-beta", "2rc1" all read as 2
fn num(part: &str) -> u32 {
    part.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0)
}

fn version_tuple(v: &str) -> (u32, u32, u32) {
    let v = v.trim().trim_start_matches('v');
    let mut it = v.split('.');
    (
        num(it.next().unwrap_or("0")),
        num(it.next().unwrap_or("0")),
        num(it.next().unwrap_or("0")),
    )
}

fn is_newer(latest: &str, current: &str) -> bool {
    version_tuple(latest) > version_tuple(current)
}

// one-shot GitHub release check. blocking (ureq), so it runs on a blocking thread. takes the newest
// published release, prereleases included. any error (offline, no releases yet, rate limited) just
// means "no update", we stay quiet
fn fetch_latest_blocking() -> Option<UpdateInfo> {
    let body = ureq::get(REPO_API)
        // github rejects requests without a user agent
        .set("User-Agent", concat!("dddioxus/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/vnd.github+json")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    // list endpoint returns an array, newest first. take the first non-draft one
    let release = json
        .as_array()?
        .iter()
        .find(|r| !r.get("draft").and_then(|d| d.as_bool()).unwrap_or(false))?;
    let tag = release.get("tag_name")?.as_str()?.to_string();
    let url = release.get("html_url")?.as_str()?.to_string();
    if is_newer(&tag, env!("CARGO_PKG_VERSION")) {
        Some(UpdateInfo { version: tag, url })
    } else {
        None
    }
}

/// A thin bar at the top that always points to the latest Cobalt test build, and additionally
/// announces a debugger update when a newer release exists on GitHub. It checks once on launch and
/// can be dismissed for the session. Place it at the app root so it shows on both the connection
/// screen and the connected workspace.
#[component]
pub fn UpdateBanner() -> Element {
    let mut info = use_signal(|| None::<UpdateInfo>);
    let mut dismissed = use_signal(|| false);
    let mut checked = use_signal(|| false);

    if !checked() {
        checked.set(true);
        spawn(async move {
            // the github call blocks, so push it to tokio's blocking pool and only touch the
            // signal back on the ui task
            let found = tokio::task::spawn_blocking(fetch_latest_blocking).await.ok().flatten();
            if found.is_some() {
                info.set(found);
            }
        });
    }

    if dismissed() {
        return rsx! {};
    }
    let update = info();
    let app_version = update.as_ref().map(|u| u.version.clone());
    let app_url = update.as_ref().map(|u| u.url.clone());

    rsx! {
        div { class: "flex items-center gap-3 px-4 py-1.5 shrink-0 bg-indigo-600 text-white text-xs shadow-sm",
            if let Some(version) = app_version {
                span { class: "font-medium", "A new debugger version ({version}) is available." }
                button {
                    class: "underline underline-offset-2 hover:text-indigo-200 cursor-pointer",
                    onclick: move |_| {
                        if let Some(u) = app_url.as_ref() {
                            // opens the release page in the default browser
                            let _ = open::that(u);
                        }
                    },
                    "Download"
                }
                span { class: "text-indigo-300", "\u{00B7}" }
            }
            span { "Don\u{2019}t forget to grab the latest Cobalt test build." }
            button {
                class: "ml-auto text-indigo-200 hover:text-white cursor-pointer",
                title: "Dismiss",
                onclick: move |_| dismissed.set(true),
                "\u{2715}"
            }
        }
    }
}
