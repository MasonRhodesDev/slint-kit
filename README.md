# slint-kit

Shared **Slint** design tokens for Mason apps, fed by
[LMTT](https://github.com/MasonRhodesDev/linux-multi-theme-toggle) Material You colors.

## Layout

| Path | Role |
|------|------|
| `ui/theme.slint` | `Theme` global (semantic + Material tokens) |
| `ui/widgets.slint` | Optional `Page` / `Panel` chrome |
| `ThemeBridge` | Load + live-reload from LMTT |

## Consumer setup

### Cargo

```toml
slint-kit = { path = "../slint-kit" }
# or git = "https://github.com/MasonRhodesDev/slint-kit"
```

### `build.rs`

```rust
fn main() {
    let mut cfg = slint_build::CompilerConfiguration::new()
        .with_include_paths(vec![slint_kit::ui_dir()]);
    slint_build::compile_with_config("ui/app.slint", cfg).unwrap();
}
```

### Root `.slint`

```slint
import { Theme } from "theme.slint";
import { Palette, ColorScheme, Button } from "std-widgets.slint";

export { Theme, Palette }

export component AppWindow inherits Window {
    background: Theme.page-bg;
}
```

### Rust

```rust
let ui = AppWindow::new()?;
let _bridge = slint_kit::ThemeBridge::attach(ui.as_weak(), |ui, tokens| {
    slint_kit::apply_theme!(ui.global::<Theme>(), tokens);
    ui.invoke_sync_palette(); // public function on root that sets Palette.color-scheme
})?;
std::mem::forget(_bridge);
ui.run()?;
```

Add on the root component:

```slint
public function sync-palette() {
    Palette.color-scheme = Theme.mode == "light" ? ColorScheme.light : ColorScheme.dark;
}
```

## LMTT

`lmtt switch` writes `~/.config/matugen/lmtt-slint.json` via the built-in `slint` module.
`ThemeBridge` watches that directory and re-applies without restarting the app.

Fallback order: `lmtt-slint.json` → `lmtt-colors.css` → embedded dark/light.

## Semantic properties

Prefer: `page-bg`, `panel-bg`, `fg`, `fg-muted`, `border`, `accent`, `on-accent`,
`danger`, `canvas-bg`, `tile-bg`, `tile-selected`.

## Vigil / overlays

Vigil keeps its interpreter theme contract. Map the same JSON keys into theme
`color-scheme` + accent when adopting LMTT there.
