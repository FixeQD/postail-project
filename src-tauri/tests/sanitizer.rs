use postail_project_lib::utils::sanitizer::{
    auto_fix_email_html, sanitize_email_html, sanitize_email_html_with_details,
    sanitize_style_attribute,
};

#[test]
fn test_sanitize_style_removes_position() {
    let style = "color: red; position: fixed; font-size: 14px";
    let result = sanitize_style_attribute(style);
    assert!(!result.cleaned_style.contains("position"));
    assert!(result.cleaned_style.contains("color"));
    assert!(result.cleaned_style.contains("font-size"));
    assert!(result.removed_properties.contains(&"position".to_string()));
}

#[test]
fn test_sanitize_style_removes_zindex() {
    let style = "z-index: 9999; margin: 10px";
    let result = sanitize_style_attribute(style);
    assert!(!result.cleaned_style.contains("z-index"));
    assert!(result.cleaned_style.contains("margin"));
}

#[test]
fn test_sanitize_style_removes_animation() {
    let style = "animation: slide 1s ease; padding: 5px";
    let result = sanitize_style_attribute(style);
    assert!(!result.cleaned_style.contains("animation"));
    assert!(result.cleaned_style.contains("padding"));
}

#[test]
fn test_sanitize_adds_font_fallback() {
    let style = "font-family: 'Custom Font'";
    let result = sanitize_style_attribute(style);
    assert!(result.cleaned_style.contains("sans-serif"));
    assert!(result.added_font_fallback);
}

#[test]
fn test_sanitize_keeps_safe_fonts() {
    let style = "font-family: Arial, Helvetica, sans-serif";
    let result = sanitize_style_attribute(style);
    assert!(!result.added_font_fallback);
}

#[test]
fn test_sanitize_html_removes_dangerous_styles() {
    let html = r#"<div style="color: blue; position: absolute;">Hello</div>"#;
    let result = sanitize_email_html(html);
    assert!(result.contains("color"));
    assert!(!result.contains("position"));
}

#[test]
fn test_sanitize_html_removes_script_tags() {
    let html = r#"<div>Hello<script>alert('xss')</script>World</div>"#;
    let result = sanitize_email_html(html);
    assert!(!result.contains("script"));
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
}

#[test]
fn test_prefixed_properties_removed() {
    let style = "-webkit-animation: test 1s; -moz-transform: rotate(45deg); color: red";
    let result = sanitize_style_attribute(style);
    assert!(!result.cleaned_style.contains("animation"));
    assert!(!result.cleaned_style.contains("transform"));
    assert!(result.cleaned_style.contains("color"));
}

#[test]
fn test_sanitize_with_details_reports_issues() {
    let html = r#"<div style="position: fixed; z-index: 100;">Test</div>"#;
    let result = sanitize_email_html_with_details(html);
    assert!(!result.issues.is_empty());
}

#[test]
fn test_preserves_safe_email_properties() {
    let style = "color: #333; background-color: white; padding: 10px; margin: 5px; font-size: 16px; text-align: center; border: 1px solid #ccc";
    let result = sanitize_style_attribute(style);
    assert!(result.cleaned_style.contains("color"));
    assert!(result.cleaned_style.contains("background-color"));
    assert!(result.cleaned_style.contains("padding"));
    assert!(result.cleaned_style.contains("margin"));
    assert!(result.cleaned_style.contains("font-size"));
    assert!(result.cleaned_style.contains("text-align"));
    assert!(result.cleaned_style.contains("border"));
    assert!(result.removed_properties.is_empty());
}

// === Auto-fix tests ===

#[test]
fn test_auto_fix_removes_google_fonts() {
    let html = r#"<style>@import url('https://fonts.googleapis.com/css2?family=Roboto:wght@300&display=swap');</style><div style="font-family: 'Roboto', sans-serif;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(!result.contains("fonts.googleapis.com"));
    assert!(!result.contains("@import"));
}

#[test]
fn test_auto_fix_removes_font_face() {
    let html = r#"<style>@font-face { font-family: 'CustomFont'; src: url('font.woff'); }</style><div>Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(!result.contains("@font-face"));
    assert!(result.contains("Hello"));
}

#[test]
fn test_auto_fix_removes_dangerous_css() {
    let html = r#"<div style="position: fixed; z-index: 999; color: red;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(!result.contains("position"));
    assert!(!result.contains("z-index"));
    assert!(result.contains("color"));
}

#[test]
fn test_auto_fix_inlines_styles() {
    let html = r#"<style>.test { color: blue; }</style><div class="test">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(result.contains("style=") || result.contains("style ="));
}

#[test]
fn test_auto_fix_adds_font_fallbacks() {
    let html = r#"<div style="font-family: 'Custom Font';">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(result.contains("sans-serif") || result.contains("Arial"));
}

#[test]
fn test_auto_fix_preserves_safe_content() {
    let html =
        r#"<div style="color: #333; padding: 10px;"><p>Hello <strong>World</strong></p></div>"#;
    let result = auto_fix_email_html(html);
    assert!(result.contains("color"));
    assert!(result.contains("padding"));
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
    assert!(result.contains("<strong>"));
}
