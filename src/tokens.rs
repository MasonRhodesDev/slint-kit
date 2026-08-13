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

/// Load tokens: JSON → CSS → embedded fallback (dark).
pub fn load_tokens() -> TokenSet {
    if let Some(path) = slint_json_path() {
        if let Ok(t) = load_json(&path) {
            return t;
        }
    }
    if let Some(path) = css_path() {
        if let Ok(t) = load_css(&path) {
            return t;
        }
    }
    embedded_fallback("dark")
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
