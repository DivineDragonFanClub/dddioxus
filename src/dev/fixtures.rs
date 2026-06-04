use crate::components::procs_view::Selection;
use crate::protocol::{
    ComponentInfo, GetComponentsResponse, GetGlobalVariablesResponse, GetProcDescsResponse,
    GetProcTreeResponse, GetSceneNameResponse, GetTransformResponse, GlobalVariable, ProcDescInfo,
    ProcNode, ProcRoot, SceneInfo, SceneNode, Vec3,
};

pub fn scene_loaded() -> GetSceneNameResponse {
    GetSceneNameResponse {
        scene_name: "MainGameScene".into(),
        scenes: vec![
            SceneInfo {
                name: "MainGameScene".into(),
                is_active: true,
                objects: vec![
                    SceneNode {
                        name: "MainCamera".into(),
                        path: "MainGameScene/MainCamera".into(),
                        active: true,
                        children: vec![],
                    },
                    SceneNode {
                        name: "World".into(),
                        path: "MainGameScene/World".into(),
                        active: true,
                        children: vec![
                            SceneNode {
                                name: "Terrain".into(),
                                path: "MainGameScene/World/Terrain".into(),
                                active: true,
                                children: vec![],
                            },
                            SceneNode {
                                name: "Units".into(),
                                path: "MainGameScene/World/Units".into(),
                                active: true,
                                children: vec![
                                    SceneNode {
                                        name: "Alear".into(),
                                        path: "MainGameScene/World/Units/Alear".into(),
                                        active: true,
                                        children: vec![],
                                    },
                                    SceneNode {
                                        name: "Framme".into(),
                                        path: "MainGameScene/World/Units/Framme".into(),
                                        active: true,
                                        children: vec![],
                                    },
                                    SceneNode {
                                        name: "Clanne".into(),
                                        path: "MainGameScene/World/Units/Clanne".into(),
                                        active: false,
                                        children: vec![],
                                    },
                                ],
                            },
                        ],
                    },
                    SceneNode {
                        name: "UI".into(),
                        path: "MainGameScene/UI".into(),
                        active: true,
                        children: vec![SceneNode {
                            name: "HUDCanvas".into(),
                            path: "MainGameScene/UI/HUDCanvas".into(),
                            active: true,
                            children: vec![],
                        }],
                    },
                ],
            },
            SceneInfo {
                name: "DontDestroyOnLoad".into(),
                is_active: false,
                objects: vec![SceneNode {
                    name: "AudioManager".into(),
                    path: "DontDestroyOnLoad/AudioManager".into(),
                    active: true,
                    children: vec![],
                }],
            },
        ],
    }
}

pub fn scene_empty() -> GetSceneNameResponse {
    GetSceneNameResponse {
        scene_name: "EmptyScene".into(),
        scenes: vec![SceneInfo {
            name: "EmptyScene".into(),
            is_active: true,
            objects: vec![],
        }],
    }
}

pub fn globals_loaded() -> GetGlobalVariablesResponse {
    GetGlobalVariablesResponse {
        variables: vec![
            GlobalVariable {
                name: "G_Gold".into(),
                kind: "int".into(),
                value: "12500".into(),
                temporary: false,
            },
            GlobalVariable {
                name: "G_BondFragments".into(),
                kind: "int".into(),
                value: "340".into(),
                temporary: false,
            },
            GlobalVariable {
                name: "G_ChapterIndex".into(),
                kind: "int".into(),
                value: "7".into(),
                temporary: false,
            },
            GlobalVariable {
                name: "G_CurrentMap".into(),
                kind: "string".into(),
                value: "M007_Fortress".into(),
                temporary: true,
            },
            GlobalVariable {
                name: "G_Difficulty".into(),
                kind: "string".into(),
                value: "Hard".into(),
                temporary: false,
            },
            GlobalVariable {
                name: "G_GameTime".into(),
                kind: "float".into(),
                value: "14237.5".into(),
                temporary: true,
            },
        ],
    }
}

pub fn globals_empty() -> GetGlobalVariablesResponse {
    GetGlobalVariablesResponse { variables: vec![] }
}

pub fn sample_global_variable() -> GlobalVariable {
    GlobalVariable {
        name: "G_Gold".into(),
        kind: "int".into(),
        value: "12500".into(),
        temporary: false,
    }
}

pub fn sample_global_variable_string() -> GlobalVariable {
    GlobalVariable {
        name: "G_PlayerName".into(),
        kind: "string".into(),
        value: "Alear".into(),
        temporary: false,
    }
}

pub fn proc_tree_loaded() -> GetProcTreeResponse {
    GetProcTreeResponse {
        roots: vec![
            ProcRoot {
                label: "Root".into(),
                children: vec![
                    ProcNode {
                        name: "MapSequence".into(),
                        hashcode: 0x1234_ABCD_u32 as i32,
                        desc_index: 0,
                        children: vec![
                            ProcNode {
                                name: "TurnController".into(),
                                hashcode: 0x2233_4455_u32 as i32,
                                desc_index: 0,
                                children: vec![ProcNode {
                                    name: "PlayerPhase".into(),
                                    hashcode: 0x6677_8899_u32 as i32,
                                    desc_index: 1,
                                    children: vec![],
                                }],
                            },
                            ProcNode {
                                name: "CameraRig".into(),
                                hashcode: 0xAABB_CCDD_u32 as i32,
                                desc_index: 0,
                                children: vec![],
                            },
                        ],
                    },
                    ProcNode {
                        name: "UIRoot".into(),
                        hashcode: 0x1111_2222,
                        desc_index: 2,
                        children: vec![ProcNode {
                            name: "HUD".into(),
                            hashcode: 0x3333_4444,
                            desc_index: 0,
                            children: vec![],
                        }],
                    },
                ],
            },
            ProcRoot {
                label: "UI".into(),
                children: vec![],
            },
        ],
    }
}

pub fn proc_descs_loaded() -> GetProcDescsResponse {
    GetProcDescsResponse {
        descs: vec![
            ProcDescInfo {
                kind: "Method".into(),
                method: Some("OnEnter".into()),
                label: None,
            },
            ProcDescInfo {
                kind: "Method".into(),
                method: Some("OnUpdate".into()),
                label: None,
            },
            ProcDescInfo {
                kind: "Label".into(),
                method: None,
                label: Some("WaitForInput".into()),
            },
            ProcDescInfo {
                kind: "Method".into(),
                method: Some("OnExit".into()),
                label: None,
            },
        ],
        desc_index: 1,
    }
}

pub fn proc_descs_empty() -> GetProcDescsResponse {
    GetProcDescsResponse {
        descs: vec![],
        desc_index: 0,
    }
}

pub fn sample_selection() -> Selection {
    Selection {
        root: "Root".into(),
        path: vec![0, 0, 0],
        name: "PlayerPhase".into(),
        hashcode: 0x6677_8899_u32 as i32,
    }
}

pub fn sample_proc_node_leaf() -> ProcNode {
    ProcNode {
        name: "WaitForInput".into(),
        hashcode: 0x1122_3344,
        desc_index: 0,
        children: vec![],
    }
}

pub fn sample_proc_node_with_children() -> ProcNode {
    ProcNode {
        name: "TurnController".into(),
        hashcode: 0x5566_7788,
        desc_index: 1,
        children: vec![
            ProcNode {
                name: "PlayerPhase".into(),
                hashcode: 0x99AA_BBCC_u32 as i32,
                desc_index: 0,
                children: vec![],
            },
            ProcNode {
                name: "EnemyPhase".into(),
                hashcode: 0xDDEE_FF00_u32 as i32,
                desc_index: 0,
                children: vec![],
            },
        ],
    }
}

pub fn transform_identity() -> GetTransformResponse {
    GetTransformResponse {
        path: "MainGameScene/World/Units/Alear".into(),
        local_position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        local_rotation: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        local_scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
    }
}

pub fn transform_nonzero() -> GetTransformResponse {
    GetTransformResponse {
        path: "MainGameScene/World/Units/Alear".into(),
        local_position: Vec3 {
            x: 14.25,
            y: 0.0,
            z: -7.5,
        },
        local_rotation: Vec3 {
            x: 0.0,
            y: 90.0,
            z: 0.0,
        },
        local_scale: Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    }
}

pub fn components_loaded() -> GetComponentsResponse {
    GetComponentsResponse {
        path: "MainGameScene/World/Units/Alear".into(),
        components: vec![
            ComponentInfo {
                index: 0,
                type_name: "UnityEngine.Transform".into(),
                enabled: None,
            },
            ComponentInfo {
                index: 1,
                type_name: "UnityEngine.SkinnedMeshRenderer".into(),
                enabled: Some(true),
            },
            ComponentInfo {
                index: 2,
                type_name: "UnityEngine.Animator".into(),
                enabled: Some(true),
            },
            ComponentInfo {
                index: 3,
                type_name: "App.UnitController".into(),
                enabled: Some(true),
            },
            ComponentInfo {
                index: 4,
                type_name: "App.DebugHighlighter".into(),
                enabled: Some(false),
            },
        ],
    }
}

pub fn components_empty() -> GetComponentsResponse {
    GetComponentsResponse {
        path: "Empty/GameObject".into(),
        components: vec![],
    }
}

pub fn sample_component_enabled() -> ComponentInfo {
    ComponentInfo {
        index: 1,
        type_name: "UnityEngine.SkinnedMeshRenderer".into(),
        enabled: Some(true),
    }
}

pub fn sample_component_disabled() -> ComponentInfo {
    ComponentInfo {
        index: 4,
        type_name: "App.DebugHighlighter".into(),
        enabled: Some(false),
    }
}

pub fn sample_component_unknown() -> ComponentInfo {
    ComponentInfo {
        index: 0,
        type_name: "UnityEngine.Transform".into(),
        enabled: None,
    }
}
