use html_transpiler::{sanitize_email_html, sanitize_email_html_with_details};

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
fn test_sanitize_with_details_reports_issues() {
    let html = r#"<div style="position: fixed; z-index: 100;">Test</div>"#;
    let result = sanitize_email_html_with_details(html);
    assert!(!result.issues.is_empty());
}

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
