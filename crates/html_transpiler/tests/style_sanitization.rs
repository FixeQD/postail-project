use html_transpiler::sanitize_style_attribute;

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
fn test_prefixed_properties_removed() {
    let style = "-webkit-animation: test 1s; -moz-transform: rotate(45deg); color: red";
    let result = sanitize_style_attribute(style);
    assert!(!result.cleaned_style.contains("animation"));
    assert!(!result.cleaned_style.contains("transform"));
    assert!(result.cleaned_style.contains("color"));
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
