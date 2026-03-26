use html_transpiler::auto_fix_email_html;

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
