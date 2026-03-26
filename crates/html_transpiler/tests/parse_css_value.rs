use html_transpiler::parse_css_value;

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
