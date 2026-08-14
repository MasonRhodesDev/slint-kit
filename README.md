# slint-kit

Shared **Slint** design tokens for Mason apps, fed by
[LMTT](https://github.com/MasonRhodesDev/linux-multi-theme-toggle) Material You colors.

## Layout

| Path | Role |
|------|------|
| `ui/theme.slint` | `Theme` global (colors, density, `optical-shift`) |
| `ui/typography.slint` | `KitDisplay` / `KitTitle` / `KitLabel` / `KitMuted` / `KitWarning` |
| `ui/layout.slint` | `KitVStack` / `KitHStack` / `KitSpacer` / `KitDivider` |
| `ui/controls.slint` | `KitButton`, `KitCheckBox`, `KitLineEdit`, `KitComboBox`, `KitAlignPad` |
| `ui/chrome.slint` | Fields, toolbar, heading stack, list rows, badge, panels |
| `ui/widgets.slint` | `Page` / `Panel` + re-exports |
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
import { KitButton, KitAlignPad } from "widgets.slint";
import { Palette, VerticalBox, HorizontalBox, ScrollView } from "std-widgets.slint";

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

Prefer colors: `page-bg`, `panel-bg`, `fg`, `fg-muted`, `border`, `accent`, `on-accent`,
`danger`, `canvas-bg`, `tile-bg`, `tile-selected`.

Prefer density: `space-xs`…`space-xl`, `control-height`, `control-height-sm`,
`control-pad-x`, `radius-sm` / `radius-md`, `font-body` / `font-label` / `font-title`.

Prefer controls from `widgets.slint` / `controls.slint` over Fluent `std-widgets`
`Button` / `CheckBox` / `ComboBox` / `LineEdit`. `KitAlignPad` is the shared
place-beside chrome (`place(0..3)` = Left / Right / Above / Below).

## Vigil / overlays

Vigil keeps its interpreter theme contract. Map the same JSON keys into theme
`color-scheme` + accent when adopting LMTT there.
