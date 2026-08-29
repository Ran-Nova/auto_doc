use super::*;

#[test]
fn parses_named_paths_in_advanced_mode() {
    let nested: Vec<NestedMeta> = Punctuated::<NestedMeta, Token![,]>::parse_terminated
        .parse_str("path = \"docs/a.md\", paths = [\"docs/b.md\"]")
        .unwrap()
        .into_iter()
        .collect();

    let config = AutoDocArgs::from_list(&nested).unwrap();

    assert_eq!(config.path.as_deref(), Some("docs/a.md"));
    assert_eq!(
        config.paths.iter().map(|s| s.value()).collect::<Vec<_>>(),
        vec!["docs/b.md"]
    );
}
