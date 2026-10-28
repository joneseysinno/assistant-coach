# Assistant Coach

A Dioxus app built on [app-shell](https://github.com/joneseysinno/JoInn-Template) (v0.1.0). The shell
provides the layout: title bar, nav rail and panes, per-document toolbar,
side and bottom rails, resizable panels and the command palette (Ctrl+K).
This crate only describes what is different about this app.

## Run

```bash
cargo run                                                          # desktop
dx serve --platform web --no-default-features --features web       # web
```

## Where things go

| File | What to change |
|---|---|
| `src/main.rs` | `spec()`: app title, title-bar actions, nav rail, document types, starting documents |
| `src/docs.rs` | `on_command`: routes commands to each document type by id prefix |
| `src/docs/example.rs` | One document type: toolbar, left/right/bottom rails, viewport, status readout |
| `src/app.css` | App styles (prefix classes with `app-`) and theme token overrides (`--as-*`) |

### Add a document type

1. Copy `src/docs/example.rs` to `src/docs/drawing.rs`, change `ID`, the label,
   icon and the command id prefix (`ex.` to something like `dw.`).
2. Add `pub mod drawing;` to `src/docs.rs` and route its prefix in `on_command`.
3. Push `docs::drawing::doc_type()` onto `s.doc_types` in `spec()`.

It then appears in the Documents pane's "+" menu, the empty state and the
command palette automatically.

### Add a nav pane

Write a `fn MyPane() -> Element` and add
`RailItem::panel("nav.mine", "Mine", icons::HISTORY, MyPane)` to `s.nav_rail`.

### Icons

`Icon` is SVG path data on a 24x24 grid, so you can define your own:
`const MY_ICON: Icon = Icon("M4 4h16v16H4z");`

## Upgrade the shell

Change `tag` on the `app-shell` line in `Cargo.toml`, then:

```bash
cargo update -p app-shell
```
