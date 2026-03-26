use html_transpiler::{auto_fix_email_html, sanitize_email_html_with_details};

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
