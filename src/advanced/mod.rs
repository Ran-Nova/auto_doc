use darling::{ast::NestedMeta, FromMeta};
use error::AdvancedError;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse::Parser, parse2 as syn_parse2, punctuated::Punctuated, Attribute, Error, Ident, ImplItem,
    Item, Lit, LitStr, Token, Type,
};

mod error;

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

        return Ok(crate::common::expand(
            paths,
            &ident,
            item,
            Vec::new(),
            is_impl,
        )?);
    }

    let config = AutoDocArgs::from_list(&nested)
        .map_err(|error| AdvancedError::Attribute(error.to_string()))?;

    let parsed_item: Item = syn_parse2(item.clone().into())?;
    validate_advanced_config(&config, &parsed_item)?;

    let mut paths = Vec::with_capacity(config.paths.len() + 1);
    if let Some(path) = config.path {
        paths.push(path);
    }
    paths.extend(config.paths.into_iter().map(|path| path.value()));

    let AdvancedItem { ident, is_impl } = advanced_item_ident(&parsed_item)?;
    let mut additional_paths = Vec::new();

    if config.members {
        let mut item_impl = match parsed_item {
            Item::Impl(item_impl) => item_impl,
            _ => {
                return Err(AdvancedError::InvalidConfiguration(
                    "auto_doc: `members = true` requires an impl block",
                ))
            }
        };

        for member in &mut item_impl.items {
            let Some(member_info) = ImplMember::from_item(member) else {
                continue;
            };

            let member_path = config
                .member_path
                .as_deref()
                .unwrap_or("docs/{type}/{member}.md")
                .replace("{type}", &ident.to_string())
                .replace("{member}", &member_info.ident.to_string())
                .replace("{kind}", member_info.kind.as_str());
            let member_files = vec![member_path];
            let (member_doc, member_paths) =
                crate::common::load_documentation(&member_files, ident.span())?;
            member_info
                .item
                .attrs_mut()
                .push(syn::parse_quote!(#[doc = #member_doc]));
            additional_paths.extend(member_paths);
        }

        let item_tokens = quote::quote!(#item_impl).into();

        return Ok(crate::common::expand(
            paths,
            &ident,
            item_tokens,
            additional_paths,
            is_impl,
        )?);
    }

    Ok(crate::common::expand(
        paths,
        &ident,
        item,
        additional_paths,
        is_impl,
    )?)
}

struct AdvancedItem {
    pub ident: Ident,
    pub is_impl: bool,
}

fn validate_advanced_config(config: &AutoDocArgs, item: &Item) -> Result<(), AdvancedError> {
    if config.member_path.is_some() && !config.members {
        return Err(AdvancedError::InvalidConfiguration(
            "auto_doc: `member_path` requires `members = true`",
        ));
    }

    if config.members && !matches!(item, Item::Impl(_)) {
        return Err(AdvancedError::InvalidConfiguration(
            "auto_doc: `members = true` requires an impl block",
        ));
    }

    Ok(())
}

fn advanced_item_ident(item: &Item) -> Result<AdvancedItem, Error> {
    match item {
        Item::Struct(item) => Ok(AdvancedItem {
            ident: item.ident.clone(),
            is_impl: false,
        }),
        Item::Enum(item) => Ok(AdvancedItem {
            ident: item.ident.clone(),
            is_impl: false,
        }),
        Item::Trait(item) => Ok(AdvancedItem {
            ident: item.ident.clone(),
            is_impl: false,
        }),
        Item::Const(item) => Ok(AdvancedItem {
            ident: item.ident.clone(),
            is_impl: false,
        }),
        Item::Static(item) => Ok(AdvancedItem {
            ident: item.ident.clone(),
            is_impl: false,
        }),
        Item::Type(item) => Ok(AdvancedItem {
            ident: item.ident.clone(),
            is_impl: false,
        }),
        Item::Fn(item) => Ok(AdvancedItem {
            ident: item.sig.ident.clone(),
            is_impl: false,
        }),
        Item::Impl(item) => match &*item.self_ty {
            Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|segment| AdvancedItem {
                    ident: segment.ident.clone(),
                    is_impl: true,
                })
                .ok_or_else(|| Error::new(Span::call_site(), "auto_doc: unsupported impl target")),
            _ => Err(Error::new(
                Span::call_site(),
                "auto_doc: unsupported impl target",
            )),
        },
        _ => Err(Error::new(
            Span::call_site(),
            "auto_doc: unsupported item type",
        )),
    }
}

#[derive(Debug, Clone, Copy)]
enum ImplMemberKind {
    Function,
    Constant,
    Type,
}

#[derive(Debug)]
struct ImplMember<'a> {
    ident: Ident,
    kind: ImplMemberKind,
    item: &'a mut ImplItem,
}

impl<'a> ImplMember<'a> {
    fn from_item(item: &'a mut ImplItem) -> Option<Self> {
        if should_skip_member(item.attrs_mut()) {
            return None;
        }

        let (ident, kind) = match item {
            ImplItem::Const(item) => (item.ident.clone(), ImplMemberKind::Constant),
            ImplItem::Fn(item) => (item.sig.ident.clone(), ImplMemberKind::Function),
            ImplItem::Type(item) => (item.ident.clone(), ImplMemberKind::Type),
            _ => return None,
        };

        Some(Self { ident, kind, item })
    }
}

/// Check attributes for `#[doc(hidden)]`
fn should_skip_member(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr
                .parse_args::<Ident>()
                .map_or(false, |ident| ident == "hidden")
    })
}

impl ImplMemberKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Constant => "constant",
            Self::Type => "type",
        }
    }
}

trait ImplItemAttrs {
    fn attrs_mut(&mut self) -> &mut Vec<Attribute>;
}

impl ImplItemAttrs for ImplItem {
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        match self {
            ImplItem::Const(item) => &mut item.attrs,
            ImplItem::Fn(item) => &mut item.attrs,
            ImplItem::Type(item) => &mut item.attrs,
            ImplItem::Macro(item) => &mut item.attrs,
            _ => panic!("auto_doc: unsupported impl item"),
        }
    }
}
