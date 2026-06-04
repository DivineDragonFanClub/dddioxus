use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    EditCommandRequest, GetCutsceneRequest, GetCutsceneResponse, JumpCutsceneRequest, RemoveCommandRequest,
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
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetCutsceneResponse, String>>);
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
    }

    let jump = move |(index, label): (i32, String)| {
        spawn(async move {
            if rpc::call(&conn, JumpCutsceneRequest { index }).await.is_ok() {
                data.with_mut(|slot| {
                    if let Some(Ok(c)) = slot.as_mut() {
                        c.current_index = index;
                        c.current_label = label;
                    }
                });
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
                if rpc::call(&conn, EditCommandRequest { label, side, index, text }).await.is_ok() {
                    if let Ok(c) = rpc::call(&conn, GetCutsceneRequest).await {
                        data.set(Some(Ok(c)));
                    }
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
                if rpc::call(&conn, RemoveCommandRequest { label, side, index }).await.is_ok() {
                    if let Ok(c) = rpc::call(&conn, GetCutsceneRequest).await {
                        data.set(Some(Ok(c)));
                    }
                }
            });
        }
    };

    let raw_for_edit = cmd.raw.clone();

    rsx! {
        div { class: "flex items-baseline gap-2 ml-10",
            if editing() {
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
