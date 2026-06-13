# UI kit

The shared component library every page composes. The goal is that building a new page is mostly
picking components and passing a few props, never hand-writing tailwind for a button or wiring up a
loading state. Preview everything live in the dev storybook under **UI Kit gallery** (`/dev/ui-kit`).

## Conventions (read this first)

These hold for every component, so once you know them you know the whole kit.

- **Attribute forwarding.** Every component forwards extra html attributes to its root element via
  `#[props(extends = GlobalAttributes)]`. So you can pass `class`, `style`, `id`, `title`,
  `data-*`, `aria-*`, `onmouseenter`, etc. on any component without it declaring that prop:

  ```rust
  Button { class: "ml-auto", title: "Save changes", "data-testid": "save", onclick, "Save" }
  Card  { class: "flex-1", /* ... */ }
  ```

  A `class` you pass is **merged** with the component's own classes (not replaced), so you add
  layout (`flex-1`, `w-56`, `shrink-0`) without losing the component's look.

- **`tone` / `variant` / `size`.** Color, fill, and density are enums with sensible defaults, the
  same vocabulary across components:
  - `Tone`: `Indigo` (default action), `Emerald` (add/confirm), `Red` (destructive), `Gray` (neutral).
  - `ButtonVariant`: `Solid` (default), `Outline`, `Ghost`.
  - `ButtonSize` / `SelectSize`: `Md` (default), `Sm`.

- **Text props take `impl Into<String>`** — pass a literal or a `format!(...)`, both work.

- **Events are `EventHandler`** props (`onclick`, `on_input`, `on_change`, `on_commit`, `on_toggle`,
  `on_select`). Pass a closure: `onclick: move |_| do_thing()`.

- **Slots are `Element`** props (`header`, `leading`, `trailing`, `actions`). Pass `rsx! { ... }`.

## Layout

| Component | What it is | Key props |
|---|---|---|
| `Page` | the padded region a page's cards live in | `row` (side-by-side vs stacked) |
| `Card` | the elevated surface (border, shadow, rounded), optional header strip | `title`, `header`, `padded`, `body_class` |
| `PanelHeader` | header strip for a sub-panel inside a card | `title`, `subtitle`, `actions` |
| `Toolbar` | a transparent row of controls | — |
| `ListRow` | one row in a list, with hover + selected states | `selected`, `onclick` |
| `Field` | a labeled row (`label` left, control right) that stacks cleanly when narrow | `label`, `label_width` |
| `SectionLabel` | the small all-caps group label | `label` |
| `EmptyState` | the one way to render loading / error / empty | `kind`, `message`, `compact` |
| `TabBar` | underline tab switcher | `tabs`, `selected`, `on_select` |

A typical page:

```rust
Page {
    Card {
        title: "World Map",
        header: rsx! {
            SearchField { value: q(), on_input: move |v| q.set(v), class: "w-56" }
            Button { onclick: move |_| refresh(), "Refresh" }
        },
        padded: false,
        div { class: "p-3",
            if loading() {
                EmptyState { kind: StateKind::Loading, message: "Loading\u{2026}" }
            } else {
                for node in nodes {
                    ListRow { selected: node.active, onclick: move |_| pick(node.id),
                        span { class: "flex-1", "{node.name}" }
                    }
                }
            }
        }
    }
}
```

A two-pane page is `Page { row: true, Card { /* list */ } ResizablePanel { /* inspector */ } }`.

## Controls

| Component | What it is | Key props |
|---|---|---|
| `Button` | the one button | `tone`, `variant`, `size`, `disabled`, `loading`, `full_width`, `leading`, `trailing`, `onclick` |
| `Select` | the one dropdown (supply `option` children, mark one `selected`) | `tone`, `size`, `on_change` |
| `Checkbox` | accent-tinted checkbox with optional label | `checked`, `label`, `on_toggle` |
| `SearchField` | filter box with a built-in clear button | `value`, `placeholder`, `on_input` |
| `EditableNumber` | click-to-edit integer (stats, levels, turns) | `value: i64`, `width`, `on_commit` |
| `EditableText` | click-to-edit string | `value: String`, `width`, `on_commit` |

```rust
Button { tone: Tone::Emerald, loading: saving(), onclick: move |_| save(), "Save" }
Button { tone: Tone::Red, variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick, "Delete" }

Field { label: "Class",
    Select { class: "w-full", on_change: move |jid| set_class(jid),
        for c in classes { option { value: "{c.jid}", selected: c.jid == current, "{c.name}" } }
    }
}

EditableNumber { value: level as i64, width: "w-14", on_commit: move |v| set_level(v) }
```

## Adding a component

1. Add `src/components/ui/<thing>.rs` following the conventions above (enum props with `#[default]`,
   `#[props(extends = GlobalAttributes)] attributes`, `..attributes` spread on the root element).
2. Re-export it from `src/components/ui/mod.rs`.
3. Add a `StorySection` for it in `src/dev/stories/ui_kit.rs` so it shows in the gallery.
