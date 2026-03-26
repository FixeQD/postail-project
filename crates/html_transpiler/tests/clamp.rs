use html_transpiler::auto_fix_email_html;

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
