use std::sync::LazyLock;

#[derive(Debug)]
pub struct PositionInfo {
    pub is_positioned: bool,
    pub position_type: String,
    pub vertical_pos: String, // "top", "bottom", "none"
    pub vertical_value: f32,
    pub horizontal_pos: String, // "left", "right", "none"
    pub horizontal_value: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub is_overlay: bool, // large decorative element like glow
}

#[derive(Debug, Clone, Default)]
pub struct StyleSanitizeResult {
    pub cleaned_style: String,
    pub removed_properties: Vec<String>,
    pub added_font_fallback: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizeIssue {
    pub property: String,
    pub reason: String,
    pub severity: IssueSeverity,
    #[serde(default = "default_count")]
    pub count: usize,
}

/// Default value for a SanitizeIssue `count`.
///
/// # Examples
///
/// ```
/// assert_eq!(default_count(), 1);
/// ```
#[allow(dead_code)]
fn default_count() -> usize {
    1
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizeResult {
    pub html: String,
    pub issues: Vec<SanitizeIssue>,
}

// Rules for pseudo-element expansion
pub struct PseudoRule {
    pub class: String,           // e.g. "checkmark"
    pub is_before: bool,         // true = ::before, false = ::after
    pub content: String,         // literal text from content:"..."
    pub style: String,           // remaining declarations as a CSS rule body
    pub class_for_style: String, // the generated class name
}

pub static IMPORT_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"@import\s+[^;]+;?"#).expect("Invalid @import regex pattern")
});

pub static FONT_FACE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"@font-face\s*\{[^}]*\}").expect("Invalid @font-face regex pattern")
});