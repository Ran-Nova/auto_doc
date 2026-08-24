use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use syn::{
    parse::Parser, punctuated::Punctuated, Error, Expr, ExprLit, Ident, Lit, MetaNameValue, Token,
};

#[derive(Debug)]
pub(crate) struct AutoDocArgs {
    pub path: Option<String>,
    pub paths: Vec<String>,
}

impl AutoDocArgs {
    pub(crate) fn from_attribute(attr: TokenStream) -> Result<Self, Error> {
        Self::from_tokens(attr.into())
    }

    pub(crate) fn from_tokens(tokens: TokenStream2) -> Result<Self, Error> {
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

pub(crate) fn string_from_meta_name_value(nv: &MetaNameValue) -> Result<String, Error> {
    match &nv.value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Ok(s.value()),
        _ => Err(Error::new(path_span(&nv.path), "expected string literal")),
    }
}

pub(crate) fn path_span(path: &syn::Path) -> Span {
    path.segments
        .first()
        .map_or_else(Span::call_site, |seg| seg.ident.span())
}

pub(crate) fn impl_auto_doc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let config = AutoDocArgs::from_attribute(attr)?;
    let mut paths = Vec::with_capacity(config.paths.len() + 1);

    if let Some(path) = config.path {
        paths.push(path);
    }

    paths.extend(config.paths);
    let ident = get_ident(&item)?;
    crate::common::expand(paths, &ident, item, Vec::new())
}

pub(crate) fn get_ident(item: &TokenStream) -> Result<Ident, Error> {
    get_ident_from_tokens(item.clone().into())
}

pub(crate) fn get_ident_from_tokens(item_tokens: TokenStream2) -> Result<Ident, Error> {
    let mut iter = item_tokens.into_iter().peekable();

    while let Some(TokenTree::Ident(ident)) = iter.next() {
        match ident.to_string().as_str() {
            "struct" | "enum" | "trait" | "fn" | "const" | "static" | "type" => {
                if let Some(TokenTree::Ident(name)) = iter.next() {
                    return Ok(name);
                }
            }
            "impl" => {
                return get_impl_ident(&mut iter)
                    .ok_or_else(|| Error::new(Span::call_site(), "auto_doc: invalid impl block"));
            }
            _ => {}
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

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) if ident == "for" => {
                return iter.find_map(|token| match token {
                    TokenTree::Ident(ident) => Some(ident),
                    _ => None,
                });
            }
            TokenTree::Ident(ident) if first_ident.is_none() => first_ident = Some(ident),
            _ => {}
        }
    }

    first_ident
}
