//! Shared Slint theme bridge for LMTT Material You tokens.
mod tokens;

pub use tokens::{
    css_path, embedded_fallback, load_css, load_json, load_tokens, matugen_dir, parse_hex,
    slint_json_path, TokenSet,
};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use slint::{ComponentHandle, Weak};

/// Absolute path to this crate's `ui/` directory (for `slint_build` import paths).
pub fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

/// Paint a generated `Theme` global from a [`TokenSet`].
///
/// The app must `export { Theme }` from its root `.slint` after importing
/// `theme.slint` from this crate's `ui/` import path.
#[macro_export]
macro_rules! apply_theme {
    ($theme:expr, $tokens:expr) => {{
        let theme = &$theme;
        let t = &$tokens;
        theme.set_mode(slint::SharedString::from(t.mode.as_str()));

        let primary = t.get("primary");
        let on_primary = t.get("on_primary");
        let primary_container = t.get("primary_container");
        let on_primary_container = t.get("on_primary_container");
        let secondary = t.get("secondary");
        let on_secondary = t.get("on_secondary");
        let tertiary = t.get("tertiary");
        let on_tertiary = t.get("on_tertiary");
        let surface = t.get_or("surface", "background");
        let on_surface = t.get_or("on_surface", "on_background");
        let surface_variant = t.get("surface_variant");
        let on_surface_variant = t.get("on_surface_variant");
        let surface_container = t.get_or("surface_container", "surface");
        let surface_container_high = t.get_or("surface_container_high", "surface_container");
        let surface_container_highest =
            t.get_or("surface_container_highest", "surface_container_high");
        let outline = t.get("outline");
        let outline_variant = t.get_or("outline_variant", "outline");
        let background = t.get_or("background", "surface");
        let on_background = t.get_or("on_background", "on_surface");
        let error = t.get("error");
        let on_error = t.get("on_error");

        theme.set_primary(primary);
        theme.set_on_primary(on_primary);
        theme.set_primary_container(primary_container);
        theme.set_on_primary_container(on_primary_container);
        theme.set_secondary(secondary);
        theme.set_on_secondary(on_secondary);
        theme.set_tertiary(tertiary);
        theme.set_on_tertiary(on_tertiary);
        theme.set_surface(surface);
        theme.set_on_surface(on_surface);
        theme.set_surface_variant(surface_variant);
        theme.set_on_surface_variant(on_surface_variant);
        theme.set_surface_container(surface_container);
        theme.set_surface_container_high(surface_container_high);
        theme.set_surface_container_highest(surface_container_highest);
        theme.set_outline(outline);
        theme.set_outline_variant(outline_variant);
        theme.set_background(background);
        theme.set_on_background(on_background);
        theme.set_error(error);
        theme.set_on_error(on_error);

        theme.set_page_bg(background);
        theme.set_panel_bg(surface_container);
        theme.set_panel_bg_high(surface_container_high);
        theme.set_fg(on_surface);
        theme.set_fg_muted(on_surface_variant);
        theme.set_border(outline_variant);
        theme.set_accent(primary);
        theme.set_on_accent(on_primary);
        theme.set_danger(error);
        theme.set_on_danger(on_error);
        theme.set_selection_bg(primary_container);
        theme.set_canvas_bg(background);
        theme.set_tile_bg(surface_container_high);
        theme.set_tile_selected(primary_container);
        theme.set_warning_fg(secondary);
    }};
}

/// Keep the watcher alive for the process lifetime.
pub struct ThemeBridge {
    _watcher: RecommendedWatcher,
}

impl ThemeBridge {
    /// Load tokens once, invoke `apply`, then watch `~/.config/matugen` for changes.
    pub fn attach<C, F>(weak: Weak<C>, apply: F) -> Result<Self, String>
    where
        C: ComponentHandle + 'static,
        F: Fn(&C, &TokenSet) + Send + Sync + 'static,
    {
        let apply = Arc::new(apply);
        {
            let tokens = load_tokens();
            if let Some(ui) = weak.upgrade() {
                apply(&ui, &tokens);
            }
        }

        let last = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));
        let weak_watch = weak.clone();
        let apply_watch = apply.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            let Ok(event) = res else {
                return;
            };
            if !event.kind.is_modify() && !event.kind.is_create() {
                return;
            }
            let relevant = event.paths.iter().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n == "lmtt-slint.json" || n == "lmtt-colors.css")
            });
            if !relevant {
                return;
            }
            {
                let mut last = last.lock();
                if last.elapsed() < Duration::from_millis(50) {
                    return;
                }
                *last = Instant::now();
            }
            let weak = weak_watch.clone();
            let apply = apply_watch.clone();
            let _ = slint::invoke_from_event_loop(move || {
                let tokens = load_tokens();
                if let Some(ui) = weak.upgrade() {
                    apply(&ui, &tokens);
                }
            });
        })
        .map_err(|e| e.to_string())?;

        if let Some(dir) = matugen_dir() {
            let _ = std::fs::create_dir_all(&dir);
            watcher
                .watch(&dir, RecursiveMode::NonRecursive)
                .map_err(|e| e.to_string())?;
        }

        Ok(Self { _watcher: watcher })
    }
}
