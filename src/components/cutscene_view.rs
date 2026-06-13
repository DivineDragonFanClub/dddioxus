use dioxus::prelude::*;

use crate::components::toast::use_toasts;
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    EditCommandRequest, GetCutsceneRequest, GetCutsceneResponse, GetSceneNameRequest, JumpCutsceneRequest,
    RemoveCommandRequest, SceneInfo, SceneNode,
};
use crate::rpc;

#[derive(Clone, PartialEq)]
struct StepCmd {
    name: String,
    args: String,
    raw: String,
}

fn parse_cmds(text: &str) -> Vec<StepCmd> {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).map(parse_cmd).collect()
}

fn parse_cmd(line: &str) -> StepCmd {
    let line = line.trim();
    let raw = line.to_string();
    let (name, args) = match line.find(['(', '（']) {
        Some(open) => {
            let name = line[..open].trim().to_string();
            let mut after = line[open..].chars();
            after.next();
            let inner = after.as_str().trim().trim_end_matches([')', '）']);
            let args = inner.split([',', '，']).map(str::trim).collect::<Vec<_>>().join(", ");
            (name, args)
        }
        None => (line.to_string(), String::new()),
    };
    StepCmd { name, args, raw }
}

fn localize_cmd(name: &str) -> &str {
    match name {
        "ウェイト" => "Wait",
        "ジャンプ" => "Jump",
        "ラベル" => "Label",
        "変数" => "Variant",
        "メッセージロード" => "Load Message",
        "章タイトル" => "Chapter Title",
        "サウンドイベント" => "Sound Event",
        "背景" => "Background",
        "背景自動" => "Background (auto)",
        "ライト" => "Light",
        "ライト自動" => "Light (auto)",
        "フェードイン" => "Fade In",
        "フェードアウト" => "Fade Out",
        "白フェードイン" => "White Fade In",
        "白フェードアウト" => "White Fade Out",
        "一枚絵表示" => "Show Picture",
        "一枚絵非表示" => "Hide Picture",
        "キャラ配置" => "Place Character",
        "キャラ配置調整" => "Adjust Character Position",
        "キャラ削除" => "Delete Character",
        "キャラ表示切替" => "Toggle Character Visibility",
        "キャラモーション再生" => "Play Character Motion",
        "キャラモーション待ち" => "Wait Character Motion",
        "キャラアニメーター切替" => "Switch Character Animator",
        "キャラ回転" => "Rotate Character",
        "キャラ視線" => "Character Gaze",
        "キャラ視線リセット" => "Reset Character Gaze",
        "キャラ武器装備" => "Equip Weapon",
        "キャラ武器装備解除" => "Unequip Weapon",
        "キャラ釣り竿装備" => "Equip Fishing Rod",
        "キャラ釣り竿装備解除" => "Unequip Fishing Rod",
        "キャラカメラ" => "Character Camera",
        "シーンカメラ" => "Scene Camera",
        "エフェクト表示" => "Show Effect",
        "エフェクト削除" => "Delete Effect",
        "テロップエフェクト表示" => "Show Telop Effect",
        "テロップエフェクト削除" => "Delete Telop Effect",
        "フェイス会話再生" => "Play Face Talk",
        "パペット会話中断" => "Pause Puppet Talk",
        "スプリットビュー作成" => "Create Split View",
        "スプリットビューアクティブ" => "Split View Active",
        "スプリットビューアニメ再生" => "Split View Play Anim",
        "スプリットビューアニメ待ち" => "Split View Wait Anim",
        "スプリットビューキャラカメラ" => "Split View Character Camera",
        "スプリットビューシーンカメラ" => "Split View Scene Camera",
        "スプリットビューカメラのみ状態開始" => "Begin Split View Camera-Only",
        "スプリットビューカメラのみ状態終了" => "End Split View Camera-Only",
        other => other,
    }
}

#[component]
pub fn CutsceneView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let toasts = use_toasts();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetCutsceneResponse, String>>);
    let mut cameras = use_signal(Vec::<String>::new);
    use_context_provider(|| CameraCatalog(cameras));
    let mut search = use_signal(String::new);
    let mut mounted = use_signal(|| false);

    let mut fetch = move || {
        if loading() {
            return;
        }
        loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetCutsceneRequest).await;
            data.set(Some(result));
            loading.set(false);
        });
    };

    if !mounted() {
        mounted.set(true);
        fetch();
        // Grab the scene's cameras once when the view opens; reused by every
        // Character Camera row's dropdown.
        spawn(async move {
            if let Ok(scene) = rpc::call(&conn, GetSceneNameRequest).await {
                cameras.set(collect_cameras(&scene.scenes));
            }
        });
    }

    let jump = move |(index, label): (i32, String)| {
        spawn(async move {
            match rpc::call(&conn, JumpCutsceneRequest { index }).await {
                Ok(_) => data.with_mut(|slot| {
                    if let Some(Ok(c)) = slot.as_mut() {
                        c.current_index = index;
                        c.current_label = label;
                    }
                }),
                Err(e) => toasts.show(format!("Jump failed: {e}")),
            }
        });
    };

    rsx! {
        div { class: "flex flex-col flex-1 min-h-0",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                h2 { class: "text-white font-bold text-sm", "Cutscene" }
                if let Some(Ok(c)) = data() {
                    if c.active {
                        span { class: "text-indigo-300 text-xs", "{c.demo_name}" }
                    }
                }
                input {
                    class: "ml-3 flex-1 px-3 py-1 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-sm",
                    placeholder: "Filter command...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button {
                    class: "text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: loading(),
                    onclick: move |_| fetch(),
                    if loading() { "Refreshing..." } else { "Refresh" }
                }
            }
            div { class: "flex-1 overflow-auto bg-gray-800 p-4 font-mono text-xs",
                match data() {
                    Some(Ok(c)) if !c.active => rsx! {
                        p { class: "text-gray-500", "No cutscene is playing right now." }
                    },
                    Some(Ok(c)) => {
                        let query = search().to_lowercase();
                        let current_label = c.current_label.clone();
                        rsx! {
                            p { class: "text-gray-500 mb-2", "{c.steps.len()} steps · {c.mess_file}" }
                            for step in c.steps.iter()
                                .filter(|s| query.is_empty()
                                    || s.label.to_lowercase().contains(&query)
                                    || s.dialogue.to_lowercase().contains(&query)
                                    || s.before.to_lowercase().contains(&query)
                                    || s.after.to_lowercase().contains(&query))
                                .cloned()
                            {
                                {
                                    let is_current = !current_label.is_empty() && step.label == current_label;
                                    let idx = step.index;
                                    let lbl = step.label.clone();
                                    rsx! {
                                        div {
                                            key: "{step.index}",
                                            class: if is_current {
                                                "flex flex-col py-0.5 px-1 rounded bg-indigo-900 text-yellow-300"
                                            } else {
                                                "flex flex-col py-0.5 px-1 rounded hover:bg-gray-800 text-gray-200"
                                            },
                                            div { class: "flex items-baseline gap-2",
                                                button {
                                                    class: "text-gray-500 hover:text-indigo-300 w-8 text-right shrink-0",
                                                    title: "Jump to this label",
                                                    onclick: move |_| jump((idx, lbl.clone())),
                                                    "{step.index}"
                                                }
                                                span { class: "text-gray-500 text-[10px] truncate", "{step.label}" }
                                                if is_current {
                                                    span { class: "ml-auto text-[10px] text-yellow-400", "← current" }
                                                }
                                            }
                                            for (j, cmd) in parse_cmds(&step.before).into_iter().enumerate() {
                                                CommandRow {
                                                    key: "b{j}",
                                                    label: step.label.clone(),
                                                    side: "before".to_string(),
                                                    index: j as i32,
                                                    cmd,
                                                    data,
                                                }
                                            }
                                            if !step.dialogue.is_empty() {
                                                div { class: "ml-10 text-gray-100 whitespace-pre-wrap", "\u{1F4AC} {step.dialogue}" }
                                            }
                                            for (j, cmd) in parse_cmds(&step.after).into_iter().enumerate() {
                                                CommandRow {
                                                    key: "a{j}",
                                                    label: step.label.clone(),
                                                    side: "after".to_string(),
                                                    index: j as i32,
                                                    cmd,
                                                    data,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! { p { class: "text-red-500", "Error: {err}" } },
                    None => rsx! { p { class: "text-gray-400", "Loading cutscene..." } },
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct CommandRowProps {
    label: String,
    side: String,
    index: i32,
    cmd: StepCmd,
    data: Signal<Option<Result<GetCutsceneResponse, String>>>,
}

#[component]
fn CommandRow(props: CommandRowProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let toasts = use_toasts();
    let mut data = props.data;
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| props.cmd.raw.clone());

    let label = props.label.clone();
    let side = props.side.clone();
    let index = props.index;
    let cmd = props.cmd.clone();
    let name_color = if side == "before" { "#6ee7b7" } else { "#c4b5fd" };

    let commit = {
        let label = label.clone();
        let side = side.clone();
        move || {
            editing.set(false);
            let (label, side, text) = (label.clone(), side.clone(), draft());
            spawn(async move {
                match rpc::call(&conn, EditCommandRequest { label, side, index, text }).await {
                    Ok(_) => {
                        if let Ok(c) = rpc::call(&conn, GetCutsceneRequest).await {
                            data.set(Some(Ok(c)));
                        }
                    }
                    Err(e) => toasts.show(format!("Edit failed: {e}")),
                }
            });
        }
    };

    let remove = {
        let label = label.clone();
        let side = side.clone();
        move |_| {
            let (label, side) = (label.clone(), side.clone());
            spawn(async move {
                match rpc::call(&conn, RemoveCommandRequest { label, side, index }).await {
                    Ok(_) => {
                        if let Ok(c) = rpc::call(&conn, GetCutsceneRequest).await {
                            data.set(Some(Ok(c)));
                        }
                    }
                    Err(e) => toasts.show(format!("Remove failed: {e}")),
                }
            });
        }
    };

    let raw_for_edit = cmd.raw.clone();
    let save_label = label.clone();
    let save_side = side.clone();

    rsx! {
        div { class: "flex items-baseline gap-2 ml-10",
            if editing() {
                if cmd.name.as_str() == "キャラカメラ" {
                    CharacterCameraEditor {
                        raw: cmd.raw.clone(),
                        on_save: move |text: String| {
                            editing.set(false);
                            let (label, side) = (save_label.clone(), save_side.clone());
                            spawn(async move {
                                match rpc::call(&conn, EditCommandRequest { label, side, index, text }).await {
                                    Ok(_) => {
                                        if let Ok(c) = rpc::call(&conn, GetCutsceneRequest).await {
                                            data.set(Some(Ok(c)));
                                        }
                                    }
                                    Err(e) => toasts.show(format!("Edit failed: {e}")),
                                }
                            });
                        },
                        on_cancel: move |_| editing.set(false),
                    }
                } else {
                    input {
                        class: "flex-1 px-1 bg-gray-900 text-gray-100 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none",
                        value: "{draft}",
                        autofocus: true,
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: {
                            let mut commit = commit.clone();
                            move |e| {
                                if e.key() == Key::Enter {
                                    commit();
                                } else if e.key() == Key::Escape {
                                    editing.set(false);
                                }
                            }
                        },
                    }
                }
            } else {
                span { class: "shrink-0", style: "color: {name_color}", title: "{cmd.name}", "{localize_cmd(&cmd.name)}" }
                if !cmd.args.is_empty() {
                    span { class: "truncate", style: "color: #22d3ee", title: "{cmd.args}", "({cmd.args})" }
                }
                button {
                    class: "ml-auto shrink-0 text-gray-600 hover:text-indigo-300",
                    title: "Edit",
                    onclick: move |_| {
                        draft.set(raw_for_edit.clone());
                        editing.set(true);
                    },
                    "✎"
                }
                button {
                    class: "shrink-0 text-gray-600 hover:text-red-400",
                    title: "Remove",
                    onclick: remove,
                    "✕"
                }
            }
        }
    }
}

/// Camera names sourced once from the live scene (children of `/…/Cameras`),
/// shared via context so each Character Camera row can populate its dropdown.
#[derive(Clone, Copy)]
struct CameraCatalog(Signal<Vec<String>>);

struct CmdParts {
    name: String,
    open: char,
    close: char,
    sep: String,
    args: Vec<String>,
}

/// Split a raw command line `name(arg, arg, …)` into its parts, preserving the
/// original bracket/comma style (full-width 　（），or half-width) so the line we
/// send back to the game re-parses identically.
fn split_cmd(raw: &str) -> Option<CmdParts> {
    let raw = raw.trim();
    let open_idx = raw.find(['(', '（'])?;
    let name = raw[..open_idx].trim().to_string();
    let open = raw[open_idx..].chars().next()?;
    let close = if open == '（' { '）' } else { ')' };
    let after_open = &raw[open_idx + open.len_utf8()..];
    let close_idx = after_open.rfind(close).unwrap_or(after_open.len());
    let inner = &after_open[..close_idx];

    let sep = match inner.find(['，', ',']) {
        Some(pos) => {
            let c = inner[pos..].chars().next().unwrap();
            let mut s = String::new();
            s.push(c);
            if inner[pos + c.len_utf8()..].starts_with(' ') {
                s.push(' ');
            }
            s
        }
        None => "，".to_string(),
    };
    let args = inner.split(['，', ',']).map(|s| s.trim().to_string()).collect();

    Some(CmdParts { name, open, close, sep, args })
}

/// Collect the camera object names found under any `Cameras` node in the scene
/// (e.g. the leaves under `/RootObject/Cameras/CharacterBase` and `…/SceneBase`).
fn collect_cameras(scenes: &[SceneInfo]) -> Vec<String> {
    fn leaves(node: &SceneNode, out: &mut Vec<String>) {
        if node.children.is_empty() {
            out.push(node.name.clone());
        } else {
            for child in &node.children {
                leaves(child, out);
            }
        }
    }
    fn find(node: &SceneNode, out: &mut Vec<String>) {
        if node.name == "Cameras" {
            for child in &node.children {
                leaves(child, out);
            }
        } else {
            for child in &node.children {
                find(child, out);
            }
        }
    }

    let mut out = Vec::new();
    for scene in scenes {
        for node in &scene.objects {
            find(node, &mut out);
        }
    }
    out.sort();
    out.dedup();
    out
}

fn arg_label(index: usize) -> &'static str {
    ["Camera", "Character", "Camera Anim"].get(index).copied().unwrap_or("Arg")
}

#[derive(PartialEq, Clone, Props)]
struct CharacterCameraEditorProps {
    raw: String,
    on_save: EventHandler<String>,
    on_cancel: EventHandler<()>,
}

/// Bespoke editor for the キャラカメラ (Character Camera) command. The first
/// argument (the camera) becomes a dropdown sourced from the live scene's
/// cameras; the remaining arguments stay free-text for now.
#[component]
fn CharacterCameraEditor(props: CharacterCameraEditorProps) -> Element {
    let cameras = use_context::<CameraCatalog>().0;

    let parts = split_cmd(&props.raw);
    let name = parts.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let open = parts.as_ref().map(|p| p.open).unwrap_or('（');
    let close = parts.as_ref().map(|p| p.close).unwrap_or('）');
    let sep = parts.as_ref().map(|p| p.sep.clone()).unwrap_or_else(|| "，".to_string());
    let init_args = parts.map(|p| p.args).unwrap_or_else(|| vec![props.raw.clone()]);

    let mut args = use_signal(|| init_args);

    let on_save = props.on_save;
    let on_cancel = props.on_cancel;

    let save = move |_| {
        let line = format!("{}{}{}{}", name, open, args().join(&sep), close);
        on_save.call(line);
    };

    let field_class = "flex-1 px-1 py-0.5 bg-gray-800 text-gray-100 rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-xs";

    rsx! {
        div { class: "flex flex-col gap-1 w-full bg-gray-900 rounded p-2 border border-gray-700",
            for i in 0..args().len() {
                {
                    let lbl = arg_label(i);
                    let current = args.read().get(i).cloned().unwrap_or_default();
                    let cams = cameras();
                    let use_dropdown = i == 0 && !cams.is_empty();
                    let show_current = use_dropdown && !current.is_empty() && !cams.contains(&current);
                    rsx! {
                        div { key: "{i}", class: "flex items-center gap-2",
                            span { class: "text-gray-500 text-[10px] w-24 shrink-0 text-right", "{lbl}" }
                            if use_dropdown {
                                select {
                                    class: "{field_class}",
                                    onchange: move |e| args.with_mut(|a| a[i] = e.value()),
                                    option { value: "", selected: current.is_empty(), "— select camera —" }
                                    if show_current {
                                        option { value: "{current}", selected: true, "{current} (not in scene)" }
                                    }
                                    for cam in cams.clone() {
                                        option { key: "{cam}", value: "{cam}", selected: cam == current, "{cam}" }
                                    }
                                }
                            } else {
                                input {
                                    class: "{field_class}",
                                    value: "{current}",
                                    oninput: move |e| args.with_mut(|a| a[i] = e.value()),
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex gap-2 justify-end mt-1",
                button {
                    class: "text-gray-400 hover:text-gray-200 text-xs px-2",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    class: "text-white bg-indigo-500 hover:bg-indigo-600 rounded text-xs px-3 py-0.5",
                    onclick: save,
                    "Save"
                }
            }
        }
    }
}
