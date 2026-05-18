//! Theme endpoints: schema-validated create + contrast reporting.
//!
//! - `POST /api/v1/themes/contrast` takes a draft tokens map and returns
//!   WCAG ratios for the canonical pairs (foreground/background,
//!   primary/background, etc.). Used by the Theme Studio to gate
//!   save, and by any other caller wanting a second opinion on
//!   whether a palette is legible.
//! - `POST /api/v1/themes` (upcoming phase) will validate against
//!   schemas/theme.schema.json and persist to
//!   <ship-dir>/themes/<id>.json. Deferred: we don't have storage
//!   yet, and Theme Studio works fine against a draft-only flow.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ContrastRequest {
    /// Token map. Keys match the names in theme.schema.json. Unknown
    /// tokens are skipped.
    pub tokens: BTreeMap<String, String>,
    /// Override minimums. Missing fields fall back to WCAG defaults.
    #[serde(default)]
    pub min_body_ratio: Option<f64>,
    #[serde(default)]
    pub min_interactive_ratio: Option<f64>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct PairReport {
    pub foreground: String,
    pub background: String,
    pub label: String,
    /// `None` when either side couldn't be parsed (e.g. unsupported format).
    pub ratio: Option<f64>,
    pub aa_body: Option<bool>,
    pub aa_large: Option<bool>,
    pub aaa_body: Option<bool>,
    pub required_min: f64,
    pub passes: Option<bool>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ContrastReport {
    pub ok: bool,
    pub pairs: Vec<PairReport>,
    pub unparsable: Vec<String>,
}

/// Canonical pairs we score. `(foreground, background, required_min_kind)`
/// where `kind` is "body" for text pairs, "interactive" for surfaces.
const PAIRS: &[(&str, &str, &str, &str)] = &[
    ("foreground", "background", "body text", "body"),
    ("muted-foreground", "background", "muted text", "body"),
    ("muted-foreground", "muted", "muted on muted", "body"),
    ("primary-foreground", "primary", "primary button label", "body"),
    ("accent-foreground", "accent", "accent label", "body"),
    ("destructive-foreground", "destructive", "destructive label", "body"),
    ("primary", "background", "primary accent", "interactive"),
    ("destructive", "background", "destructive accent", "interactive"),
    ("success", "background", "success accent", "interactive"),
    ("warning", "background", "warning accent", "interactive"),
    ("border", "background", "border", "interactive"),
];

#[utoipa::path(
    operation_id = "themeContrast",
    post,
    path = "/api/v1/themes/contrast",
    request_body = ContrastRequest,
    responses(
        (status = 200, description = "Contrast report", body = ContrastReport),
    ),
    tag = "framework"
)]
pub async fn theme_contrast(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<ContrastRequest>,
) -> axum::response::Response {
    let body_min = req.min_body_ratio.unwrap_or(4.5);
    let interactive_min = req.min_interactive_ratio.unwrap_or(3.0);

    let mut unparsable = Vec::new();
    let mut resolve = |name: &str| -> Option<(f64, f64, f64)> {
        let raw = req.tokens.get(name)?;
        match parse_color(raw) {
            Some(rgb) => Some(rgb),
            None => {
                if !unparsable.iter().any(|n: &String| n == name) {
                    unparsable.push(name.to_string());
                }
                None
            }
        }
    };

    let mut pairs = Vec::new();
    let mut all_ok = true;

    for (fg, bg, label, kind) in PAIRS {
        let fg_rgb = resolve(fg);
        let bg_rgb = resolve(bg);
        let required = if *kind == "body" { body_min } else { interactive_min };
        let (ratio, passes, aa_body, aa_large, aaa_body) = match (fg_rgb, bg_rgb) {
            (Some(f), Some(b)) => {
                let r = contrast_ratio(f, b);
                let pass = r >= required;
                if !pass { all_ok = false; }
                (
                    Some(r),
                    Some(pass),
                    Some(r >= 4.5),
                    Some(r >= 3.0),
                    Some(r >= 7.0),
                )
            }
            _ => (None, None, None, None, None),
        };
        pairs.push(PairReport {
            foreground: fg.to_string(),
            background: bg.to_string(),
            label: label.to_string(),
            ratio,
            aa_body,
            aa_large,
            aaa_body,
            required_min: required,
            passes,
        });
    }

    (
        StatusCode::OK,
        Json(ContrastReport {
            ok: all_ok && unparsable.is_empty(),
            pairs,
            unparsable,
        }),
    )
        .into_response()
}

// ── Color parsing + contrast math ────────────────────────────────────

/// Parse a CSS color into (R, G, B) in the 0..=1 range. Supports hex,
/// rgb(), hsl(), and oklch()/oklab(). Alpha is dropped — contrast
/// against partial transparency isn't something WCAG defines for our
/// purposes, and the Theme Studio warns separately.
fn parse_color(raw: &str) -> Option<(f64, f64, f64)> {
    let t = raw.trim();
    if let Some(rest) = t.strip_prefix('#') {
        return parse_hex(rest);
    }
    if let Some(inner) = strip_fn(t, "rgba") {
        return parse_rgb_like(inner);
    }
    if let Some(inner) = strip_fn(t, "rgb") {
        return parse_rgb_like(inner);
    }
    if let Some(inner) = strip_fn(t, "hsla") {
        return parse_hsl_like(inner);
    }
    if let Some(inner) = strip_fn(t, "hsl") {
        return parse_hsl_like(inner);
    }
    if let Some(inner) = strip_fn(t, "oklch") {
        return parse_oklch(inner);
    }
    if let Some(inner) = strip_fn(t, "oklab") {
        return parse_oklab(inner);
    }
    None
}

fn strip_fn<'a>(s: &'a str, name: &str) -> Option<&'a str> {
    let lower = s.to_ascii_lowercase();
    if !lower.starts_with(name) {
        return None;
    }
    let after = &s[name.len()..].trim_start();
    let after = after.strip_prefix('(')?;
    let inner = after.strip_suffix(')')?;
    Some(inner)
}

fn parse_hex(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.trim();
    let (r, g, b) = match hex.len() {
        3 => (
            u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
        ),
        4 => (
            u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
        ),
        6 | 8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        ),
        _ => return None,
    };
    Some((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
}

fn split_args(inner: &str) -> Vec<&str> {
    // Accept space-separated modern syntax + comma-separated legacy.
    let normalized = inner.replace('/', " ");
    let raw: Vec<&str> = normalized.split(|c: char| c == ',' || c.is_whitespace()).collect();
    let _ = raw;
    inner
        .split(|c: char| c == ',' || c.is_whitespace() || c == '/')
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_component(s: &str, max: f64) -> Option<f64> {
    let t = s.trim();
    if let Some(rest) = t.strip_suffix('%') {
        Some(rest.parse::<f64>().ok()? / 100.0 * max)
    } else {
        t.parse::<f64>().ok()
    }
}

fn parse_rgb_like(inner: &str) -> Option<(f64, f64, f64)> {
    let parts = split_args(inner);
    if parts.len() < 3 { return None; }
    let r = parse_component(parts[0], 255.0)?;
    let g = parse_component(parts[1], 255.0)?;
    let b = parse_component(parts[2], 255.0)?;
    Some((r / 255.0, g / 255.0, b / 255.0))
}

fn parse_hsl_like(inner: &str) -> Option<(f64, f64, f64)> {
    let parts = split_args(inner);
    if parts.len() < 3 { return None; }
    let h = parts[0]
        .trim_end_matches("deg")
        .parse::<f64>()
        .ok()?
        .rem_euclid(360.0);
    let s = parts[1].trim_end_matches('%').parse::<f64>().ok()? / 100.0;
    let l = parts[2].trim_end_matches('%').parse::<f64>().ok()? / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hh = h / 60.0;
    let x = c * (1.0 - (hh.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hh as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Some((r1 + m, g1 + m, b1 + m))
}

/// oklch(L C H) with L in 0..1 (or 0-100 with % suffix), C in 0..0.4,
/// H in degrees.
fn parse_oklch(inner: &str) -> Option<(f64, f64, f64)> {
    let parts = split_args(inner);
    if parts.len() < 3 { return None; }
    let l = parts[0]
        .strip_suffix('%')
        .map(|s| s.parse::<f64>().map(|v| v / 100.0))
        .unwrap_or_else(|| parts[0].parse::<f64>())
        .ok()?;
    let c = parts[1].parse::<f64>().ok()?;
    let h_deg = parts[2].trim_end_matches("deg").parse::<f64>().ok()?;
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    oklab_to_srgb(l, a, b)
}

fn parse_oklab(inner: &str) -> Option<(f64, f64, f64)> {
    let parts = split_args(inner);
    if parts.len() < 3 { return None; }
    let l = parts[0]
        .strip_suffix('%')
        .map(|s| s.parse::<f64>().map(|v| v / 100.0))
        .unwrap_or_else(|| parts[0].parse::<f64>())
        .ok()?;
    let a = parts[1].parse::<f64>().ok()?;
    let b = parts[2].parse::<f64>().ok()?;
    oklab_to_srgb(l, a, b)
}

fn oklab_to_srgb(l: f64, a: f64, b: f64) -> Option<(f64, f64, f64)> {
    // Björn Ottosson's inverse transform. l, a, b → linear sRGB →
    // gamma → sRGB. Values are clamped to [0,1] since out-of-gamut is
    // fine for contrast purposes.
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let bb = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;
    Some((
        linear_to_srgb(r).clamp(0.0, 1.0),
        linear_to_srgb(g).clamp(0.0, 1.0),
        linear_to_srgb(bb).clamp(0.0, 1.0),
    ))
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(rgb: (f64, f64, f64)) -> f64 {
    let (r, g, b) = (
        srgb_to_linear(rgb.0),
        srgb_to_linear(rgb.1),
        srgb_to_linear(rgb.2),
    );
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses_with_short_and_long_forms() {
        let a = parse_color("#fff").unwrap();
        assert!((a.0 - 1.0).abs() < 1e-9);
        let b = parse_color("#ffffff").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn contrast_white_on_black_is_21() {
        let w = parse_color("#ffffff").unwrap();
        let k = parse_color("#000000").unwrap();
        let r = contrast_ratio(w, k);
        assert!((r - 21.0).abs() < 0.01, "got {}", r);
    }

    #[test]
    fn oklch_parses() {
        // oklch(100% 0 0) should be near-white.
        let w = parse_color("oklch(1 0 0)").unwrap();
        assert!(w.0 > 0.95 && w.1 > 0.95 && w.2 > 0.95);
    }
}
