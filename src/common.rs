use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::quote;
#[cfg(feature = "advanced")]
use quote::quote_spanned;
use std::{
    env::var,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
#[cfg(feature = "advanced")]
use syn::parse::Parser;
#[cfg(feature = "advanced")]
use syn::Attribute;
use syn::{Error, Ident};

pub(crate) fn expand(
    paths: Vec<String>,
    ident: &Ident,
    item: TokenStream,
    is_impl: bool,
    source: Option<&str>,
) -> Result<TokenStream, Error> {
    let files = if paths.is_empty() {
        if is_impl {
            vec![]
        } else {
            vec![format!(
                "{}/{ident}.md",
                source
                    .filter(|value| !value.is_empty())
                    .map_or_else(|| "docs".to_owned(), |value| format!("docs/{value}"))
            )]
        }
    } else {
        paths
    };

    let documentation = if files.is_empty() {
        LoadedDocumentation::default()
    } else {
        load_documentation(&files, ident.span())?
    }
    .contents;

    let input_tokens: TokenStream2 = item.into();

    let doc_attr = if files.is_empty() {
        quote! {}
    } else {
        documentation_attribute_tokens(&files, &documentation)
    };

    let has_section = |section: &str| {
        documentation
            .iter()
            .any(|content| content.lines().any(|line| line.trim() == section))
    };

    let errors_lint_attr =
        has_section("# Errors").then(|| quote! { #[allow(clippy::missing_errors_doc)] });
    let panics_lint_attr =
        has_section("# Panics").then(|| quote! { #[allow(clippy::missing_panics_doc)] });
    let safety_lint_attr =
        has_section("# Safety").then(|| quote! { #[allow(clippy::missing_safety_doc)] });

    Ok(quote! {
        #errors_lint_attr
        #panics_lint_attr
        #safety_lint_attr
        #doc_attr
        #input_tokens
    }
    .into())
}

#[cfg(feature = "advanced")]
pub(crate) fn documentation_attribute(
    files: &[String],
    contents: &[String],
    span: Span,
) -> Attribute {
    let joined_files = files.join(", ");
    let documentation = Literal::string(&format!(
        "Documentation pulled from: `{joined_files}`\n\n{}",
        contents.join("\n\n")
    ));

    Attribute::parse_outer
        .parse2(quote_spanned! {
        span=> #[doc = #documentation]
        })
        .expect("auto_doc: generated documentation attribute should parse")
        .into_iter()
        .next()
        .expect("auto_doc: generated documentation attribute is missing")
}

fn documentation_attribute_tokens(files: &[String], contents: &[String]) -> TokenStream2 {
    let joined_files = files.join(", ");
    let documentation = Literal::string(&format!(
        "Documentation pulled from: `{joined_files}`\n\n{}",
        contents.join("\n\n")
    ));

    quote! {
        #[doc = #documentation]
    }
}

#[derive(Default)]
pub(crate) struct LoadedDocumentation {
    pub contents: Vec<String>,
}

pub(crate) fn load_documentation(
    files: &[String],
    span: Span,
) -> Result<LoadedDocumentation, Error> {
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let base_path = Path::new(&manifest_dir);
    let mut contents = Vec::with_capacity(files.len());

    for file in files {
        let full_path = if Path::new(file).is_absolute() {
            PathBuf::from(file)
        } else {
            base_path.join(file)
        };

        let content = fs::read_to_string(&full_path).map_err(|error| {
            let detail = if error.kind() == ErrorKind::NotFound {
                format!(
                    "auto_doc: file not found at `{file}`. You can use `source = \"folder-name\"` to set the base path."
                )
            } else {
                format!("auto_doc: cannot read file `{file}`: {error}")
            };
            Error::new(span, detail)
        })?;

        contents.push(content);
    }

    Ok(LoadedDocumentation { contents })
}
