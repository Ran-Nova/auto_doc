use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{env::var, path::Path};
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
        let mut args = AutoDocArgs {
            path: None,
            paths: Vec::new(),
        };

        let tokens: TokenStream2 = attr.into();

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

    let mut files = Vec::new();

    if let Some(path) = config.path {
        files.push(path);
    }

    files.extend(config.paths);

    if files.is_empty() {
        files.push(format!("docs/{ident}.md"));
    }

    let mut doc_contents: Vec<String> = Vec::new();
    let mut final_absolute_paths = Vec::new();

    for file in &files {
        let full_path = base_path.join(file);

        if !full_path.exists() {
            return Err(Error::new(
                span,
                format!("auto_doc: file not found at `{file}`"),
            ));
        }

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| Error::new(span, format!("auto_doc: cannot read file `{file}`: {e}")))?;

        doc_contents.push(content);

        let abs_path = full_path
            .to_str()
            .ok_or_else(|| Error::new(span, format!("auto_doc: non-UTF8 path `{file}`")))?;

        final_absolute_paths.push(abs_path.to_owned());
    }

    let mut full_markdown = format!("📖 Documentation pulled from: `{}`\n\n", files.join(", "));

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
    let mut iter = item_tokens.into_iter().peekable();

    for tt in iter.by_ref() {
        if let TokenTree::Ident(ident) = &tt {
            match ident.to_string().as_str() {
                "struct" | "enum" | "trait" | "fn" => {
                    if let Some(TokenTree::Ident(name)) = iter.next() {
                        return Ok(name);
                    }

                    break;
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
