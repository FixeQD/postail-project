//! Shared types for the sanitizer pipeline.

use std::sync::LazyLock;

/// Spatial position of a CSS-positioned element, used by the table layout stage.
#[derive(Debug)]
pub struct PositionInfo {
    pub is_positioned: bool,
    pub position_type: String,
    /// `"top"`, `"bottom"`, or `"none"`
    pub vertical_pos: String,
    pub vertical_value: f32,
    /// `"left"`, `"right"`, or `"none"`
    pub horizontal_pos: String,
    pub horizontal_value: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    /// Large decorative element (glow, blob) — rendered inline rather than in a corner cell.
    pub is_overlay: bool,
}

/// Result of sanitizing a single `style=""` attribute.
#[derive(Debug, Clone, Default)]
pub struct StyleSanitizeResult {
    pub cleaned_style: String,
    pub removed_properties: Vec<String>,
    pub added_font_fallback: bool,
}

/// Severity of a sanitization issue reported to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
}

/// A single sanitization issue (aggregated by property + reason in the pipeline).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizeIssue {
    pub property: String,
    pub reason: String,
    pub severity: IssueSeverity,
    #[serde(default = "default_count")]
    pub count: usize,
}

#[allow(dead_code)]
fn default_count() -> usize {
    1
}

/// Summary of what the sanitizer changed vs. the original HTML.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct HtmlDiff {
    pub removed_tags: Vec<String>,
    pub removed_attributes: Vec<(String, String)>,
    pub modified_styles: Vec<(String, Vec<String>)>,
}

/// Full result returned to the frontend after sanitization.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizeResult {
    pub html: String,
    pub issues: Vec<SanitizeIssue>,
    pub diff: HtmlDiff,
}

/// A resolved `::before` / `::after` pseudo-element rule.
pub struct PseudoRule {
    pub class: String,
    pub is_before: bool,
    pub content: String,
    pub style: String,
    pub class_for_style: String,
}

pub static IMPORT_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"@import\s+[^;]+;?"#).expect("invalid @import regex"));

pub static FONT_FACE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"@font-face\s*\{[^}]*\}").expect("invalid @font-face regex")
});
