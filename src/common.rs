use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::quote;
use std::{
    env::var,
    fs,
    path::{Path, PathBuf},
};
use syn::{Error, Ident};

pub(crate) fn expand(
    paths: Vec<String>,
    ident: &Ident,
    item: TokenStream,
    additional_paths: Vec<String>,
    is_impl: bool,
) -> Result<TokenStream, Error> {
    let files = if paths.is_empty() {
        if is_impl {
            vec![]
        } else {
            vec![format!("docs/{ident}.md")]
        }
    } else {
        paths
    };

    let (full_markdown, mut absolute_paths) = if files.is_empty() {
        (String::new(), Vec::new())
    } else {
        load_documentation(&files, ident.span())?
    };

    absolute_paths.extend(additional_paths);

    let input_tokens: TokenStream2 = item.into();

    let doc_attr = if full_markdown.is_empty() {
        quote! {}
    } else {
        let total_doc_lit = Literal::string(&full_markdown);

        quote! { #[doc = #total_doc_lit] }
    };

    let include_block = if absolute_paths.is_empty() {
        quote! {}
    } else {
        quote! {
            const _: () = {
                #( const _: &str = include_str!(#absolute_paths); )*
            };
        }
    };

    Ok(quote! {
        #doc_attr
        #input_tokens
        #include_block
    }
    .into())
}

pub(crate) fn load_documentation(
    files: &[String],
    span: Span,
) -> Result<(String, Vec<String>), Error> {
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let base_path = Path::new(&manifest_dir);
    let mut contents = Vec::with_capacity(files.len());
    let mut absolute_paths = Vec::with_capacity(files.len());

    for file in files {
        let full_path = if Path::new(file).is_absolute() {
            PathBuf::from(file)
        } else {
            base_path.join(file)
        };

        let content = fs::read_to_string(&full_path).map_err(|error| {
            let detail = if error.kind() == std::io::ErrorKind::NotFound {
                format!("auto_doc: file not found at `{file}`")
            } else {
                format!("auto_doc: cannot read file `{file}`: {error}")
            };
            Error::new(span, detail)
        })?;

        contents.push(content);

        absolute_paths.push(
            full_path
                .to_str()
                .ok_or_else(|| Error::new(span, format!("auto_doc: non-UTF8 path `{file}`")))?
                .to_owned(),
        );
    }

    let joined_files = files.join(", ");
    let total_len = contents.iter().map(String::len).sum::<usize>();

    let mut markdown = String::with_capacity(joined_files.len() + total_len + 40);

    markdown.push_str("📖 Documentation pulled from: `");
    markdown.push_str(&joined_files);
    markdown.push_str("`\n\n");

    for content in contents {
        markdown.push_str(&content);
        markdown.push_str("\n\n");
    }

    Ok((markdown, absolute_paths))
}
