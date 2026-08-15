//! Token loading: LMTT JSON → CSS → embedded fallbacks.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use slint::Color;

#[derive(Debug, Clone, Deserialize)]
struct LmttFile {
    mode: String,
    colors: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TokenSet {
    pub mode: String,
    pub colors: HashMap<String, Color>,
}

impl TokenSet {
    pub fn is_dark(&self) -> bool {
        !self.mode.eq_ignore_ascii_case("light")
    }

    pub fn get(&self, key: &str) -> Color {
        self.colors
            .get(key)
            .copied()
            .unwrap_or_else(|| Color::from_rgb_u8(0x12, 0x13, 0x1a))
    }

    pub fn get_or(&self, key: &str, fallback_key: &str) -> Color {
        self.colors
            .get(key)
            .or_else(|| self.colors.get(fallback_key))
            .copied()
            .unwrap_or_else(|| Color::from_rgb_u8(0x12, 0x13, 0x1a))
    }
}

pub fn matugen_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("matugen"))
}

pub fn slint_json_path() -> Option<PathBuf> {
    matugen_dir().map(|d| d.join("lmtt-slint.json"))
}

pub fn css_path() -> Option<PathBuf> {
    matugen_dir().map(|d| d.join("lmtt-colors.css"))
}

pub fn system_matugen_dir() -> PathBuf {
    PathBuf::from("/etc/matugen")
}

/// Load tokens: user JSON → system JSON → user CSS → system CSS → embedded.
pub fn load_tokens() -> TokenSet {
    load_tokens_preferring("dark")
}

/// Same search as [`load_tokens`], but the embedded fallback follows `mode`
/// (`"light"` / `"dark"` / empty → dark). Used when no LMTT file is present.
pub fn load_tokens_preferring(mode: &str) -> TokenSet {
    let json_paths = slint_json_path()
        .into_iter()
        .chain(std::iter::once(system_matugen_dir().join("lmtt-slint.json")));
    for path in json_paths {
        if let Ok(t) = load_json(&path) {
            return t;
        }
    }
    let css_paths = css_path()
        .into_iter()
        .chain(std::iter::once(system_matugen_dir().join("lmtt-colors.css")));
    for path in css_paths {
        if let Ok(t) = load_css(&path) {
            return t;
        }
    }
    embedded_fallback(if mode.is_empty() { "dark" } else { mode })
}

/// System paths only: `/etc/matugen/lmtt-slint.json` → `lmtt-colors.css` → embedded.
/// Does not read `$HOME` or `dirs::config_dir()`. For pre-login consumers (greeter).
pub fn load_tokens_system(mode: &str) -> TokenSet {
    let json = system_matugen_dir().join("lmtt-slint.json");
    if let Ok(t) = load_json(&json) {
        return t;
    }
    let css = system_matugen_dir().join("lmtt-colors.css");
    if let Ok(t) = load_css(&css) {
        return t;
    }
    embedded_fallback(if mode.is_empty() { "dark" } else { mode })
}

/// Slint `Theme` color properties (kebab-case) painted from a [`TokenSet`].
/// Same mapping as `apply_theme!`.
pub fn kit_color_bindings(tokens: &TokenSet) -> Vec<(&'static str, Color)> {
    let primary = tokens.get("primary");
    let on_primary = tokens.get("on_primary");
    let primary_container = tokens.get("primary_container");
    let on_primary_container = tokens.get("on_primary_container");
    let secondary = tokens.get("secondary");
    let on_secondary = tokens.get("on_secondary");
    let tertiary = tokens.get("tertiary");
    let on_tertiary = tokens.get("on_tertiary");
    let surface = tokens.get_or("surface", "background");
    let on_surface = tokens.get_or("on_surface", "on_background");
    let surface_variant = tokens.get("surface_variant");
    let on_surface_variant = tokens.get("on_surface_variant");
    let surface_container = tokens.get_or("surface_container", "surface");
    let surface_container_high = tokens.get_or("surface_container_high", "surface_container");
    let surface_container_highest =
        tokens.get_or("surface_container_highest", "surface_container_high");
    let outline = tokens.get("outline");
    let outline_variant = tokens.get_or("outline_variant", "outline");
    let background = tokens.get_or("background", "surface");
    let on_background = tokens.get_or("on_background", "on_surface");
    let error = tokens.get("error");
    let on_error = tokens.get("on_error");
    vec![
        ("primary", primary),
        ("on-primary", on_primary),
        ("primary-container", primary_container),
        ("on-primary-container", on_primary_container),
        ("secondary", secondary),
        ("on-secondary", on_secondary),
        ("tertiary", tertiary),
        ("on-tertiary", on_tertiary),
        ("surface", surface),
        ("on-surface", on_surface),
        ("surface-variant", surface_variant),
        ("on-surface-variant", on_surface_variant),
        ("surface-container", surface_container),
        ("surface-container-high", surface_container_high),
        ("surface-container-highest", surface_container_highest),
        ("outline", outline),
        ("outline-variant", outline_variant),
        ("background", background),
        ("on-background", on_background),
        ("error", error),
        ("on-error", on_error),
        ("page-bg", background),
        ("panel-bg", surface_container),
        ("panel-bg-high", surface_container_high),
        ("fg", on_surface),
        ("fg-muted", on_surface_variant),
        ("border", outline_variant),
        ("accent", primary),
        ("on-accent", on_primary),
        ("danger", error),
        ("on-danger", on_error),
        ("selection-bg", primary_container),
        ("canvas-bg", background),
        ("tile-bg", surface_container_high),
        ("tile-selected", primary_container),
        ("warning-fg", secondary),
    ]
}

pub fn load_json(path: &Path) -> Result<TokenSet, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: LmttFile = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut colors = HashMap::new();
    for (k, v) in file.colors {
        if let Some(c) = parse_hex(&v) {
            colors.insert(k, c);
        }
    }
    Ok(TokenSet {
        mode: file.mode,
        colors,
    })
}

pub fn load_css(path: &Path) -> Result<TokenSet, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut colors = HashMap::new();
    let mut mode = "dark".to_string();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("/* Mode:") {
            let m = rest.trim().trim_end_matches("*/").trim();
            if !m.is_empty() {
                mode = m.to_string();
            }
        }
        // @define-color primary #9fd491;
        if let Some(rest) = line.strip_prefix("@define-color ") {
            let rest = rest.trim().trim_end_matches(';');
            let mut parts = rest.splitn(2, char::is_whitespace);
            let Some(name) = parts.next() else { continue };
            let Some(value) = parts.next() else { continue };
            let value = value.trim();
            if value.starts_with('@') {
                continue; // alias — skip
            }
            if let Some(c) = parse_hex(value) {
                colors.insert(name.to_string(), c);
            }
        }
    }
    if colors.is_empty() {
        return Err("no colors in css".into());
    }
    Ok(TokenSet { mode, colors })
}

pub fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::from_rgb_u8(r, g, b))
}

pub fn embedded_fallback(mode: &str) -> TokenSet {
    let dark = mode != "light";
    let pairs: &[(&str, &str)] = if dark {
        &[
            ("primary", "#9fd491"),
            ("on_primary", "#003a03"),
            ("primary_container", "#22511c"),
            ("on_primary_container", "#bbf0aa"),
            ("secondary", "#edb8cd"),
            ("on_secondary", "#4a2532"),
            ("tertiary", "#bbc3fa"),
            ("on_tertiary", "#1e2a5a"),
            ("surface", "#12131a"),
            ("on_surface", "#e3e1ec"),
            ("surface_variant", "#44464f"),
            ("on_surface_variant", "#c5c5d6"),
            ("surface_container", "#1e1f27"),
            ("surface_container_high", "#292931"),
            ("surface_container_highest", "#33343c"),
            ("outline", "#8f909f"),
            ("outline_variant", "#44464f"),
            ("background", "#12131a"),
            ("on_background", "#e3e1ec"),
            ("error", "#ffb4ab"),
            ("on_error", "#690005"),
        ]
    } else {
        &[
            ("primary", "#3a6a33"),
            ("on_primary", "#ffffff"),
            ("primary_container", "#bbf0aa"),
            ("on_primary_container", "#003a03"),
            ("secondary", "#7d525f"),
            ("on_secondary", "#ffffff"),
            ("tertiary", "#555d8f"),
            ("on_tertiary", "#ffffff"),
            ("surface", "#fbf8ff"),
            ("on_surface", "#1a1b23"),
            ("surface_variant", "#e0e2ec"),
            ("on_surface_variant", "#44464f"),
            ("surface_container", "#efedf4"),
            ("surface_container_high", "#e9e7ef"),
            ("surface_container_highest", "#e3e1ec"),
            ("outline", "#74767f"),
            ("outline_variant", "#c5c6d0"),
            ("background", "#fbf8ff"),
            ("on_background", "#1a1b23"),
            ("error", "#ba1a1a"),
            ("on_error", "#ffffff"),
        ]
    };
    let mut colors = HashMap::new();
    for (k, v) in pairs {
        if let Some(c) = parse_hex(v) {
            colors.insert((*k).to_string(), c);
        }
    }
    TokenSet {
        mode: if dark { "dark" } else { "light" }.into(),
        colors,
    }
}
