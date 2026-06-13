//! The shared UI kit. Every page composes these instead of hand-rolling buttons, inputs, and
//! panels, so the whole app stays consistent. See `README.md` in this folder for the full guide,
//! and preview everything live in the dev storybook ("UI Kit gallery", `/dev/ui-kit`).
//!
//! Two conventions cover the whole kit:
//! - every component forwards extra html attributes to its root via `#[props(extends =
//!   GlobalAttributes)]`, so you can pass `class` (merged, not replaced), `style`, `id`, `title`,
//!   `data-*`, `aria-*`, event handlers, etc. on any component.
//! - color / fill / size are the shared enums [`Tone`], [`ButtonVariant`], [`ButtonSize`] with
//!   sensible defaults.
pub mod button;
pub mod card;
pub mod field;
pub mod layout;
pub mod select;
pub mod tabs;

pub use button::{Button, ButtonSize, ButtonVariant, Tone};
pub use card::{Card, PanelHeader};
pub use field::{Checkbox, EditableNumber, EditableText, SearchField};
pub use layout::{EmptyState, Field, ListRow, Page, SectionLabel, StateKind};
pub use select::{Select, SelectSize, SelectTone};
pub use tabs::TabBar;
