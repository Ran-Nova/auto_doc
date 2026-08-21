use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    env::var,
    fs,
    path::{Path, PathBuf},
};
use syn::{
    parse::Parser, punctuated::Punctuated, Error, Expr, ExprLit, Ident, Lit, MetaNameValue, Token,
};

#[derive(Debug)]
struct AutoDocArgs {
    pub path: Option<String>,
    pub paths: Vec<String>,
}

impl AutoDocArgs {
    fn from_attribute(attr: TokenStream) -> Result<Self, Error> {
        Self::from_tokens(attr.into())
    }

    fn from_tokens(tokens: TokenStream2) -> Result<Self, Error> {
        let mut args = AutoDocArgs {
            path: None,
            paths: Vec::new(),
        };

        if tokens.is_empty() {
            return Ok(args);
        }

        if let Ok(exprs) = Punctuated::<Expr, Token![,]>::parse_terminated.parse2(tokens.clone()) {
            let mut only_strings = true;

            for expr in &exprs {
                if !matches!(
                    expr,
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(_),
                        ..
                    })
                ) {
                    only_strings = false;
                    break;
                }
            }

            if only_strings {
                for expr in exprs {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = expr
                    {
                        args.paths.push(s.value());
                    }
                }
                return Ok(args);
            }
        }

        let metas = Punctuated::<MetaNameValue, Token![,]>::parse_terminated.parse2(tokens)?;

        for nv in metas {
            if nv.path.is_ident("path") {
                if args
                    .path
                    .replace(string_from_meta_name_value(&nv)?)
                    .is_some()
                {
                    return Err(Error::new(path_span(&nv.path), "duplicate `path` argument"));
                }
            } else if nv.path.is_ident("paths") {
                args.paths.push(string_from_meta_name_value(&nv)?);
            } else {
                return Err(Error::new(path_span(&nv.path), "unknown auto_doc argument"));
            }
        }

        Ok(args)
    }
}

fn string_from_meta_name_value(nv: &MetaNameValue) -> Result<String, Error> {
    match &nv.value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Ok(s.value()),
        _ => Err(Error::new(path_span(&nv.path), "expected string literal")),
    }
}

fn path_span(path: &syn::Path) -> Span {
    path.segments
        .first()
        .map_or_else(Span::call_site, |seg| seg.ident.span())
}

/// Automatically generates documentation for the given item based on the
/// provided attributes.
///
/// Supported forms:
/// - `#[auto_doc]`
/// - `#[auto_doc(path = "docs/Item.md") ]`
/// - `#[auto_doc("docs/Item.md", "docs/Other.md") ]`
/// - `#[auto_doc(paths = "docs/A.md", paths = "docs/B.md") ]`
///
/// If no paths are provided, the macro falls back to `docs/<ItemName>.md`.
#[proc_macro_attribute]
pub fn auto_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_auto_doc(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn impl_auto_doc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let base_path = Path::new(&manifest_dir);

    let config = AutoDocArgs::from_attribute(attr)?;
    let ident = get_ident(&item)?;
    let span = ident.span();

    let mut files = Vec::with_capacity(config.paths.len() + 1);

    if let Some(path) = config.path {
        files.push(path);
    }

    files.extend(config.paths);

    if files.is_empty() {
        files.push(format!("docs/{ident}.md"));
    }

    let mut doc_contents: Vec<String> = Vec::with_capacity(files.len());
    let mut final_absolute_paths = Vec::with_capacity(files.len());

    for file in &files {
        let full_path = if Path::new(file).is_absolute() {
            PathBuf::from(file)
        } else {
            base_path.join(file)
        };

        let content = fs::read_to_string(&full_path).map_err(|e| {
            let detail = if e.kind() == std::io::ErrorKind::NotFound {
                format!("auto_doc: file not found at `{file}`")
            } else {
                format!("auto_doc: cannot read file `{file}`: {e}")
            };

            Error::new(span, detail)
        })?;

        doc_contents.push(content);

        let abs_path = full_path
            .to_str()
            .ok_or_else(|| Error::new(span, format!("auto_doc: non-UTF8 path `{file}`")))?;

        final_absolute_paths.push(abs_path.to_owned());
    }

    let joined_files = files.join(", ");
    let content_total_len = doc_contents.iter().map(String::len).sum::<usize>();
    let mut full_markdown = String::with_capacity(joined_files.len() + content_total_len + 40);

    full_markdown.push_str("📖 Documentation pulled from: `");
    full_markdown.push_str(&joined_files);
    full_markdown.push_str("`\n\n");

    for content in doc_contents {
        full_markdown.push_str(&content);
        full_markdown.push_str("\n\n");
    }

    let total_doc_lit = Literal::string(&full_markdown);
    let input_tokens: TokenStream2 = item.into();

    Ok(quote! {
        #[doc = #total_doc_lit]
        #input_tokens

        const _: () = {
            #( const _: &str = include_str!(#final_absolute_paths); )*
        };
    }
    .into())
}

fn get_ident(item: &TokenStream) -> Result<Ident, Error> {
    let item_tokens: TokenStream2 = item.clone().into();
    get_ident_from_tokens(item_tokens)
}

fn get_ident_from_tokens(item_tokens: TokenStream2) -> Result<Ident, Error> {
    let mut iter = item_tokens.into_iter().peekable();

    while let Some(tt) = iter.next() {
        if let TokenTree::Ident(ident) = tt {
            match ident.to_string().as_str() {
                "struct" | "enum" | "trait" | "fn" | "const" | "static" | "type" => {
                    if let Some(TokenTree::Ident(name)) = iter.next() {
                        return Ok(name);
                    }

                    break;
                }
                "impl" => {
                    return get_impl_ident(&mut iter).ok_or_else(|| {
                        Error::new(Span::call_site(), "auto_doc: invalid impl block")
                    });
                }
                _ => (),
            }
        }
    }

    Err(Error::new(
        Span::call_site(),
        "auto_doc: unsupported item type",
    ))
}

fn get_impl_ident<I>(iter: &mut std::iter::Peekable<I>) -> Option<Ident>
where
    I: Iterator<Item = TokenTree>,
{
    let mut first_ident = None;

    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Ident(ident) if ident == "for" => {
                return iter.find_map(|tt| {
                    if let TokenTree::Ident(ident) = tt {
                        Some(ident)
                    } else {
                        None
                    }
                });
            }

            TokenTree::Ident(ident) if first_ident.is_none() => {
                first_ident = Some(ident);
            }

            _ => {}
        }
    }

    first_ident
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
