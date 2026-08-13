use slint::ComponentHandle;
use slint_kit::{apply_theme, ThemeBridge};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = Gallery::new()?;
    let bridge = ThemeBridge::attach(ui.as_weak(), |ui, tokens| {
        apply_theme!(ui.global::<Theme>(), tokens);
        ui.invoke_sync_palette();
    })?;
    std::mem::forget(bridge);
    ui.run()?;
    Ok(())
}
