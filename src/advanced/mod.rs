use crate::common::expand;
use darling::{ast::NestedMeta, FromMeta};
use error::AdvancedError;
use item::{advanced_item_ident, AdvancedItem};
use members::load_members;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parser, parse2 as syn_parse2, punctuated::Punctuated, Error, Item, Lit, LitStr, Token,
};

mod error;
mod item;
mod members;

#[derive(Debug, FromMeta)]
struct AutoDocArgs {
    #[darling(default)]
    path: Option<String>,
    #[darling(default)]
    paths: Vec<LitStr>,
    #[darling(default)]
    members: bool,
    #[darling(default)]
    member_path: Option<String>,
    #[darling(default)]
    source: Option<String>,
}

pub(crate) fn impl_auto_doc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    expand_auto_doc(attr, item).map_err(AdvancedError::into_syn_error)
}

fn expand_auto_doc(attr: TokenStream, item: TokenStream) -> Result<TokenStream, AdvancedError> {
    let nested: Vec<NestedMeta> = Punctuated::<NestedMeta, Token![,]>::parse_terminated
        .parse2(attr.into())?
        .into_iter()
        .collect();

    if nested
        .iter()
        .all(|meta| matches!(meta, NestedMeta::Lit(Lit::Str(_))))
    {
        let paths = nested
            .into_iter()
            .map(|meta| match meta {
                NestedMeta::Lit(Lit::Str(path)) => path.value(),
                _ => unreachable!(),
            })
            .collect();

        let parsed_item: Item = syn_parse2(item.clone().into())?;
        let AdvancedItem { ident, is_impl } = advanced_item_ident(&parsed_item)?;

        return Ok(expand(paths, &ident, item, is_impl, None)?);
    }

    let config = AutoDocArgs::from_list(&nested)
        .map_err(|error| AdvancedError::Attribute(error.to_string()))?;

    let parsed_item: Item = syn_parse2(item.clone().into())?;

    validate_advanced_config(&config, &parsed_item)?;

    let mut paths = Vec::with_capacity(config.paths.len() + 1);

    if let Some(path) = config.path.as_ref() {
        paths.push(path.clone());
    }

    paths.extend(config.paths.iter().map(LitStr::value));

    let AdvancedItem { ident, is_impl } = advanced_item_ident(&parsed_item)?;
    if config.members {
        let mut parsed_item = parsed_item;

        load_members(&mut parsed_item, &ident, &config)?;

        let item_tokens = quote!(#parsed_item).into();

        return Ok(expand(
            paths,
            &ident,
            item_tokens,
            is_impl,
            config.source.as_deref(),
        )?);
    }

    Ok(expand(
        paths,
        &ident,
        item,
        is_impl,
        config.source.as_deref(),
    )?)
}

fn validate_advanced_config(config: &AutoDocArgs, item: &Item) -> Result<(), AdvancedError> {
    if config.member_path.is_some() && !config.members {
        return Err(AdvancedError::InvalidConfiguration(
            "auto_doc: `member_path` requires `members`",
        ));
    }

    if config.members
        && !matches!(
            item,
            Item::Struct(_) | Item::Impl(_) | Item::Trait(_) | Item::Enum(_)
        )
    {
        return Err(AdvancedError::InvalidConfiguration(
            "auto_doc: `members` requires a struct, impl, trait, or enum",
        ));
    }

    Ok(())
}
