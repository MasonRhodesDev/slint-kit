//! Token loading through `lmtt-core`. Suite apps do not open theme files.
use std::collections::HashMap;

use lmtt_core::tokens::{load_preferring, load_system};
use lmtt_core::{ColorScheme, ThemeMode};
use slint::Color;

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

fn theme_mode(mode: &str) -> ThemeMode {
    if mode.eq_ignore_ascii_case("light") {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    }
}

fn token_set_from_scheme(scheme: ColorScheme) -> TokenSet {
    let mut colors = HashMap::new();
    for (k, v) in scheme.colors {
        if let Some(c) = parse_hex(&v) {
            colors.insert(k, c);
        }
    }
    TokenSet {
        mode: scheme.mode.to_string(),
        colors,
    }
}

/// Load tokens via lmtt-core (user data, then system/packaged/embedded).
pub fn load_tokens() -> TokenSet {
    load_tokens_preferring("dark")
}

pub fn load_tokens_preferring(mode: &str) -> TokenSet {
    token_set_from_scheme(load_preferring(theme_mode(mode)))
}

/// System/packaged/embedded only. Does not read the user tree.
pub fn load_tokens_system(mode: &str) -> TokenSet {
    token_set_from_scheme(load_system(theme_mode(mode)))
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
    let mode = theme_mode(mode);
    token_set_from_scheme(ColorScheme {
        mode,
        colors: lmtt_core::fallback::fallback_colors(mode),
    })
}
