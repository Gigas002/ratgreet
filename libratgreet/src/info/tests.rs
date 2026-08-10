use std::collections::HashMap;

use super::{expand_source, parse_os_release};

#[test]
fn parse_os_release_strips_quotes_and_comments() {
    let contents = "NAME=\"CachyOS Linux\"\nPRETTY_NAME=CachyOS\n# a comment\n\nID='cachyos'\n";

    let fields = parse_os_release(contents);

    assert_eq!(fields.get("NAME").map(String::as_str), Some("CachyOS Linux"));
    assert_eq!(fields.get("PRETTY_NAME").map(String::as_str), Some("CachyOS"));
    assert_eq!(fields.get("ID").map(String::as_str), Some("cachyos"));
}

#[test]
fn expand_source_replaces_named_field() {
    let mut fields = HashMap::new();
    fields.insert("PRETTY_NAME".to_string(), "CachyOS".to_string());

    let issue = expand_source("\\S{PRETTY_NAME} \\r (\\l)", &fields);

    assert_eq!(issue, "CachyOS \\r (\\l)");
}

#[test]
fn expand_source_bare_uses_name_field() {
    let mut fields = HashMap::new();
    fields.insert("NAME".to_string(), "CachyOS Linux".to_string());

    let issue = expand_source("\\S \\r", &fields);

    assert_eq!(issue, "CachyOS Linux \\r");
}

#[test]
fn expand_source_bare_falls_back_to_linux_when_missing() {
    let issue = expand_source("\\S \\r", &HashMap::new());

    assert_eq!(issue, "Linux \\r");
}

#[test]
fn expand_source_unknown_field_expands_to_nothing() {
    let issue = expand_source("[\\S{DOES_NOT_EXIST}]", &HashMap::new());

    assert_eq!(issue, "[]");
}
