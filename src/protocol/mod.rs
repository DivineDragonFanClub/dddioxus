use serde::{Deserialize, Serialize};

pub mod components;
pub mod cutscene;
pub mod force;
pub mod gamedata;
pub mod globals;
pub mod god;
pub mod inventory;
pub mod map;
pub mod mess;
pub mod procs;
pub mod scene;
pub mod script;
pub mod skill;
pub mod transform;
pub mod unit;

pub use components::*;
pub use cutscene::*;
pub use force::*;
pub use gamedata::*;
pub use globals::*;
pub use god::*;
pub use inventory::*;
pub use map::*;
pub use mess::*;
pub use procs::*;
pub use scene::*;
pub use script::*;
pub use skill::*;
pub use transform::*;
pub use unit::*;

pub(crate) const ENGAGE_NS: u16 = 1;
pub(crate) const VARIABLES_NS: u16 = 2;
pub(crate) const PROCS_NS: u16 = 3;
pub(crate) const MESS_NS: u16 = 4;
pub(crate) const GAMEDATA_NS: u16 = 5;
pub(crate) const FORCE_NS: u16 = 6;
pub(crate) const UNIT_NS: u16 = 7;
pub(crate) const INVENTORY_NS: u16 = 8;
pub(crate) const SKILL_NS: u16 = 9;
pub(crate) const MAP_NS: u16 = 10;
pub(crate) const SCRIPT_NS: u16 = 11;
pub(crate) const GOD_NS: u16 = 12;
pub(crate) const CUTSCENE_NS: u16 = 13;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OkResponse {
    pub ok: bool,
}
