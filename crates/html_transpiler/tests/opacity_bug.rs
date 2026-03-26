use html_transpiler::auto_fix_email_html;

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
