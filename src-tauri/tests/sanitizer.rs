use postail_project_lib::utils::sanitizer::{
    auto_fix_email_html, parse_css_value, sanitize_email_html, sanitize_email_html_with_details,
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

// === Opacity bug fix tests ===

#[test]
fn test_opacity_zero_is_fixed_for_fade_animations() {
    let html = r#"<div style="opacity: 0; animation: fadeIn 1s forwards;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(
        result.contains("opacity: 1"),
        "opacity: 0 should become opacity: 1 when animation is stripped. Got: {}",
        result
    );
}

#[test]
fn test_opacity_fractional_not_corrupted() {
    let html = r#"<div style="opacity: 0.35;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(
        result.contains("opacity: 0.35"),
        "opacity: 0.35 must not be corrupted. Got: {}",
        result
    );
    assert!(
        !result.contains("opacity: 1.35"),
        "opacity: 0.35 must NOT become 1.35. Got: {}",
        result
    );
}

#[test]
fn test_opacity_zero_point_five_not_corrupted() {
    let html = r#"<div style="opacity: 0.5; animation: fadeIn 1s forwards;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(
        !result.contains("opacity: 1.5"),
        "opacity: 0.5 must NOT become 1.5. Got: {}",
        result
    );
}

#[test]
fn test_opacity_zero_point_zero_is_fixed() {
    let html = r#"<div style="opacity: 0.0; animation: fadeIn 1s forwards;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(
        result.contains("opacity: 1"),
        "opacity: 0.0 should become opacity: 1. Got: {}",
        result
    );
}

#[test]
fn test_opacity_without_animation_stays() {
    let html = r#"<div style="opacity: 0;">Hello</div>"#;
    let result = auto_fix_email_html(html);
    // No fade animation -> opacity: 0 should stay as-is (intentional hiding)
    assert!(
        result.contains("opacity: 0"),
        "opacity: 0 without animation should stay. Got: {}",
        result
    );
}

// === clamp() resolution tests ===

#[test]
fn test_clamp_resolved_to_middle_value() {
    let html = r#"<div style="font-size: clamp(1rem, 2rem, 3rem);">Hello</div>"#;
    let result = auto_fix_email_html(html);
    // clamp(1rem, 2rem, 3rem) -> preferred is 2rem -> 32px
    assert!(
        !result.contains("clamp"),
        "clamp() should be resolved. Got: {}",
        result
    );
    assert!(
        result.contains("32px"),
        "2rem should become 32px. Got: {}",
        result
    );
}

#[test]
fn test_clamp_with_viewport_units() {
    let html = r#"<div style="font-size: clamp(5rem, 14vw, 11rem);">Hello</div>"#;
    let result = auto_fix_email_html(html);
    assert!(
        !result.contains("clamp"),
        "clamp() should be resolved. Got: {}",
        result
    );
    // 14vw * 6 = 84px
    assert!(
        result.contains("84px"),
        "14vw should become 84px. Got: {}",
        result
    );
}

// === parse_css_value tests ===

#[test]
fn test_parse_css_value_rem() {
    assert_eq!(parse_css_value("2rem"), 32.0);
    assert_eq!(parse_css_value("1.5rem"), 24.0);
}

#[test]
fn test_parse_css_value_px() {
    assert_eq!(parse_css_value("120px"), 120.0);
    assert_eq!(parse_css_value("-50px"), -50.0);
}

#[test]
fn test_parse_css_value_percent() {
    assert_eq!(parse_css_value("50%"), 50.0);
}

// === DANGEROUS_CSS_PROPS fix test ===

#[test]
fn test_css_values_not_treated_as_properties() {
    let style = "display: block; color: red";
    let result = sanitize_style_attribute(style);
    let pos_style = "position: fixed; color: red";
    let pos_result = sanitize_style_attribute(pos_style);
    assert!(!pos_result.cleaned_style.contains("position"));
    assert!(pos_result.cleaned_style.contains("color"));
    assert!(result.cleaned_style.contains("display"));
}

// === Flexbox conversion test ===

#[test]
fn test_flexbox_column_centering() {
    let html = r#"<div style="display: flex; flex-direction: column; align-items: center; justify-content: center;"><h1>Title</h1><p>Text</p></div>"#;
    let result = auto_fix_email_html(html);
    // Flexbox should be converted to a table
    assert!(
        result.contains("<table"),
        "Flexbox should become table layout. Got: {}",
        result
    );
    assert!(
        result.contains("Title"),
        "Content should be preserved. Got: {}",
        result
    );
    assert!(
        result.contains("Text"),
        "Content should be preserved. Got: {}",
        result
    );
}

// === Details/issues reporting test ===

#[test]
fn test_details_report_flexbox_conversion() {
    let html = r#"<div style="display: flex; flex-direction: column; align-items: center;"><p>Content</p></div>"#;
    let result = sanitize_email_html_with_details(html);
    let has_flex_issue = result
        .issues
        .iter()
        .any(|i| i.property.contains("flex") || i.reason.contains("lex"));
    assert!(
        has_flex_issue,
        "Should report flexbox conversion issue. Issues: {:?}",
        result.issues
    );
}
