use darling::{ast::NestedMeta, FromMeta};
use error::AdvancedError;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse::Parser, punctuated::Punctuated, Error, LitStr, Token};

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
        .all(|meta| matches!(meta, NestedMeta::Lit(syn::Lit::Str(_))))
    {
        let paths = nested
            .into_iter()
            .map(|meta| match meta {
                NestedMeta::Lit(syn::Lit::Str(path)) => path.value(),
                _ => unreachable!(),
            })
            .collect();

        let parsed_item: syn::Item = syn::parse2(item.clone().into())?;
        let ident = advanced_item_ident(&parsed_item)?;
        return Ok(crate::common::expand(paths, &ident, item, Vec::new())?);
    }

    let config = AutoDocArgs::from_list(&nested)
        .map_err(|error| AdvancedError::Attribute(error.to_string()))?;

    let mut paths = Vec::with_capacity(config.paths.len() + 1);
    if let Some(path) = config.path {
        paths.push(path);
    }
    paths.extend(config.paths.into_iter().map(|path| path.value()));

    let parsed_item: syn::Item = syn::parse2(item.into())?;
    let ident = advanced_item_ident(&parsed_item)?;
    let mut additional_paths = Vec::new();

    if config.member_path.is_some() && !config.members {
        return Err(AdvancedError::InvalidConfiguration(
            "auto_doc: `member_path` requires `members = true`",
        ));
    }

    if config.members {
        let mut item_impl = match parsed_item {
            syn::Item::Impl(item_impl) => item_impl,
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
        )?);
    }

    Ok(crate::common::expand(
        paths,
        &ident,
        quote::quote!(#parsed_item).into(),
        additional_paths,
    )?)
}

fn advanced_item_ident(item: &syn::Item) -> Result<syn::Ident, Error> {
    match item {
        syn::Item::Struct(item) => Ok(item.ident.clone()),
        syn::Item::Enum(item) => Ok(item.ident.clone()),
        syn::Item::Trait(item) => Ok(item.ident.clone()),
        syn::Item::Fn(item) => Ok(item.sig.ident.clone()),
        syn::Item::Const(item) => Ok(item.ident.clone()),
        syn::Item::Static(item) => Ok(item.ident.clone()),
        syn::Item::Type(item) => Ok(item.ident.clone()),
        syn::Item::Impl(item) => match item.self_ty.as_ref() {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.clone())
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
    ident: syn::Ident,
    kind: ImplMemberKind,
    item: &'a mut syn::ImplItem,
}

impl<'a> ImplMember<'a> {
    fn from_item(item: &'a mut syn::ImplItem) -> Option<Self> {
        let (ident, kind) = match item {
            syn::ImplItem::Const(item) => (item.ident.clone(), ImplMemberKind::Constant),
            syn::ImplItem::Fn(item) => (item.sig.ident.clone(), ImplMemberKind::Function),
            syn::ImplItem::Type(item) => (item.ident.clone(), ImplMemberKind::Type),
            _ => return None,
        };

        Some(Self { ident, kind, item })
    }
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
    fn attrs_mut(&mut self) -> &mut Vec<syn::Attribute>;
}

impl ImplItemAttrs for syn::ImplItem {
    fn attrs_mut(&mut self) -> &mut Vec<syn::Attribute> {
        match self {
            syn::ImplItem::Const(item) => &mut item.attrs,
            syn::ImplItem::Fn(item) => &mut item.attrs,
            syn::ImplItem::Type(item) => &mut item.attrs,
            syn::ImplItem::Macro(item) => &mut item.attrs,
            _ => panic!("auto_doc: unsupported impl item"),
        }
    }
}
