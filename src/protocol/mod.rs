pub mod components;
pub mod globals;
pub mod mess;
pub mod procs;
pub mod scene;
pub mod transform;

pub use components::*;
pub use globals::*;
pub use mess::*;
pub use procs::*;
pub use scene::*;
pub use transform::*;

// Server-owned namespaces live in 1..=255; these IDs must match the ones the server
// registers with (see `cobalt/src/lib.rs`). 256+ is reserved for runtime plugins.
pub(crate) const ENGAGE_NS: u16 = 1;
pub(crate) const VARIABLES_NS: u16 = 2;
pub(crate) const PROCS_NS: u16 = 3;
pub(crate) const MESS_NS: u16 = 4;
