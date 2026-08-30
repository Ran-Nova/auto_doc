use proc_macro::TokenStream;
use proc_macro2::{Group, Span, TokenStream as TokenStream2, TokenTree};
use std::iter::Peekable;
use syn::{parse::Parser, punctuated::Punctuated, Error, Expr, ExprLit, Ident, Lit, Token};

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

            if only_strings && !exprs.is_empty() {
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

        let mut iter = tokens.into_iter().peekable();

        parse_args_from_iter(&mut iter, &mut args)?;

        Ok(args)
    }
}

fn parse_args_from_iter<I>(iter: &mut Peekable<I>, args: &mut AutoDocArgs) -> Result<(), Error>
where
    I: Iterator<Item = TokenTree>,
{
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => {
                let key = ident.to_string();

                if let Some(TokenTree::Punct(punct)) = iter.peek() {
                    if punct.as_char() == '=' {
                        let _ = iter.next();
                    } else {
                        return Err(Error::new(ident.span(), "expected `=` after argument name"));
                    }
                } else {
                    return Err(Error::new(ident.span(), "expected `=` after argument name"));
                }

                match key.as_str() {
                    "path" => {
                        if let Some(TokenTree::Literal(lit)) = iter.next() {
                            let s = lit.to_string();
                            if s.starts_with('"') && s.ends_with('"') {
                                if args.path.is_some() {
                                    return Err(Error::new(
                                        ident.span(),
                                        "duplicate `path` argument",
                                    ));
                                }
                                args.path = Some(s.trim_matches('"').to_string());
                            } else {
                                return Err(Error::new(lit.span(), "expected string literal"));
                            }
                        } else {
                            return Err(Error::new(
                                ident.span(),
                                "expected string literal for `path`",
                            ));
                        }
                    }
                    "paths" => {
                        if let Some(next_token) = iter.next() {
                            match next_token {
                                TokenTree::Literal(lit) => {
                                    let s = lit.to_string();
                                    if s.starts_with('"') && s.ends_with('"') {
                                        args.paths.push(s.trim_matches('"').to_string());
                                    } else {
                                        return Err(Error::new(
                                            lit.span(),
                                            "expected string literal",
                                        ));
                                    }
                                }
                                TokenTree::Group(group)
                                    if group.delimiter() == proc_macro2::Delimiter::Bracket =>
                                {
                                    parse_string_array(&group, &mut args.paths)?;
                                }
                                _ => {
                                    return Err(Error::new(
                                        next_token.span(),
                                        "expected string literal or array for `paths`",
                                    ));
                                }
                            }
                        }
                    }
                    _ => return Err(Error::new(ident.span(), "unknown auto_doc argument")),
                }

                if let Some(TokenTree::Punct(punct)) = iter.peek() {
                    if punct.as_char() == ',' {
                        let _ = iter.next();
                    }
                }
            }
            TokenTree::Punct(punct) if punct.as_char() == ',' => {}
            _ => {
                return Err(Error::new(
                    token.span(),
                    "expected argument name (path or paths)",
                ))
            }
        }
    }
    Ok(())
}

pub(crate) fn impl_auto_doc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let config = AutoDocArgs::from_attribute(attr)?;
    let mut paths = Vec::with_capacity(config.paths.len() + 1);

    if let Some(path) = config.path {
        paths.push(path);
    }

    paths.extend(config.paths);

    if has_impl_keyword(&item) {
        return Err(Error::new(
            Span::call_site(),
            "auto_doc: `impl` blocks require the 'advanced' feature with `members = true`",
        ));
    }

    let ident = get_ident(&item)?;

    crate::common::expand(paths, &ident, item, Vec::new(), false)
}

pub(crate) fn get_ident(item: &TokenStream) -> Result<Ident, Error> {
    get_ident_from_tokens(item.clone().into())
}

pub(crate) fn get_ident_from_tokens(item_tokens: TokenStream2) -> Result<Ident, Error> {
    let mut iter = item_tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match token {
            // Skip proc-macro
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if let Some(&TokenTree::Group(_)) = iter.peek() {
                    let _ = iter.next();
                }
            }
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                // Skip pub/pub(item)
                "pub" => {
                    if let Some(&TokenTree::Group(_)) = iter.peek() {
                        let _ = iter.next();
                    }
                }
                // Supported item
                "struct" | "enum" | "trait" | "fn" | "const" | "static" | "type" => {
                    if let Some(TokenTree::Ident(name)) = iter.next() {
                        return Ok(name);
                    }
                }
                // Unsupported impl, need use advanced feature + members = true
                "impl" => {
                    return Err(Error::new(
                        Span::call_site(),
                        "auto_doc: `impl` blocks are only supported in 'advanced' mode with `members = true`",
                    ));
                }
                _ => {}
            },
            _ => {}
        }
    }

    Err(Error::new(
        Span::call_site(),
        "auto_doc: unsupported item type",
    ))
}

fn parse_string_array(group: &Group, output_paths: &mut Vec<String>) -> Result<(), Error> {
    for token in group.stream() {
        match token {
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if s.starts_with('"') && s.ends_with('"') {
                    output_paths.push(s.trim_matches('"').to_string());
                } else {
                    return Err(Error::new(
                        lit.span(),
                        "expected string literal inside array",
                    ));
                }
            }
            TokenTree::Punct(punct) if punct.as_char() == ',' => {}
            _ => {
                return Err(Error::new(
                    token.span(),
                    "expected string literal inside array",
                ));
            }
        }
    }
    Ok(())
}

fn has_impl_keyword(item: &TokenStream) -> bool {
    let tokens: TokenStream2 = item.clone().into();

    tokens.into_iter().any(|token| {
        if let TokenTree::Ident(ident) = token {
            ident == "impl"
        } else {
            false
        }
    })
}
