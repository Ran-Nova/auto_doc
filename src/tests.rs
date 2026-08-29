use crate::default::*;
use quote::quote;

#[test]
fn parses_string_lists() {
    let tokens = quote!("docs/a.md", "docs/b.md");
    let args = AutoDocArgs::from_tokens(tokens).unwrap();

    assert!(args.path.is_none());
    assert_eq!(args.paths, vec!["docs/a.md", "docs/b.md"]);
}

#[test]
fn parses_named_arguments() {
    let tokens = quote!(path = "docs/a.md", paths = "docs/b.md");
    let args = AutoDocArgs::from_tokens(tokens).unwrap();

    assert_eq!(args.path.as_deref(), Some("docs/a.md"));
    assert_eq!(args.paths, vec!["docs/b.md"]);
}

#[test]
fn detects_const_name() {
    let tokens = quote!(
        const ANSWER: u32 = 42;
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "ANSWER");
}

#[test]
fn detects_type_name() {
    let tokens = quote!(
        type Answer = u32;
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "Answer");
}

#[test]
fn detects_static_name() {
    let tokens = quote!(
        static ANSWER: u32 = 42;
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "ANSWER");
}

#[test]
fn detects_impl_name() {
    let tokens = quote!(
        impl MyType for TraitName {}
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "TraitName");
}

#[test]
fn detects_impl_self_name() {
    let tokens = quote!(
        impl MyType {}
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "MyType");
}

#[test]
fn detects_trait_impl_target_name() {
    let tokens = quote!(
        impl SomeTrait for MyType {}
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "MyType");
}

#[test]
fn detects_generic_impl_target_name() {
    let tokens = quote!(
        impl<T> SomeTrait<T> for MyType<T> {}
    );

    let ident = get_ident_from_tokens(tokens).unwrap();

    assert_eq!(ident.to_string(), "MyType");
}
