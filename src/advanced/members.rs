use super::AutoDocArgs;
use crate::common::load_documentation;
use proc_macro2::Ident;
use syn::{parse_quote, Attribute, Error, ImplItem, Item, Meta, TraitItem, Variant};

#[derive(Debug, Clone, Copy)]
enum MemberKind {
    Field,
    Function,
    Constant,
    Type,
    Variant,
}

pub(crate) fn load_members(
    item: &mut Item,
    ident: &Ident,
    config: &AutoDocArgs,
    additional_paths: &mut Vec<String>,
) -> Result<(), Error> {
    match item {
        Item::Struct(item_struct) => {
            for (index, member) in item_struct.fields.iter_mut().enumerate() {
                if should_skip_member(&member.attrs) {
                    continue;
                }

                let member_name = member
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| index.to_string());
                load_member_documentation(
                    ident,
                    &member_name,
                    MemberKind::Field,
                    &mut member.attrs,
                    config,
                    additional_paths,
                )?;
            }
        }
        Item::Impl(item_impl) => {
            for member in &mut item_impl.items {
                let Some(member_info) = ImplMember::from_item(member) else {
                    continue;
                };

                load_member_documentation(
                    ident,
                    &member_info.ident.to_string(),
                    member_info.kind,
                    member_info.item.attrs_mut(),
                    config,
                    additional_paths,
                )?;
            }
        }
        Item::Trait(item_trait) => {
            for member in &mut item_trait.items {
                let Some(member_info) = TraitMember::from_item(member) else {
                    continue;
                };

                load_member_documentation(
                    ident,
                    &member_info.ident.to_string(),
                    member_info.kind,
                    member_info.item.attrs_mut(),
                    config,
                    additional_paths,
                )?;
            }
        }
        Item::Enum(item_enum) => {
            for member in &mut item_enum.variants {
                let Some(member_info) = EnumMember::from_item(member) else {
                    continue;
                };

                load_member_documentation(
                    ident,
                    &member_info.ident.to_string(),
                    member_info.kind,
                    &mut member_info.item.attrs,
                    config,
                    additional_paths,
                )?;
            }
        }
        _ => {
            return Err(Error::new(
                ident.span(),
                "auto_doc: `members = true` requires a struct, impl, trait, or enum",
            ))
        }
    }

    Ok(())
}

#[derive(Debug)]
struct ImplMember<'a> {
    ident: Ident,
    kind: MemberKind,
    item: &'a mut ImplItem,
}

impl<'a> ImplMember<'a> {
    fn from_item(item: &'a mut ImplItem) -> Option<Self> {
        if should_skip_member(item.attrs_mut()) {
            return None;
        }

        let (ident, kind) = match item {
            ImplItem::Const(item) => (item.ident.clone(), MemberKind::Constant),
            ImplItem::Fn(item) => (item.sig.ident.clone(), MemberKind::Function),
            ImplItem::Type(item) => (item.ident.clone(), MemberKind::Type),
            _ => return None,
        };

        Some(Self { ident, kind, item })
    }
}

#[derive(Debug)]
struct TraitMember<'a> {
    ident: Ident,
    kind: MemberKind,
    item: &'a mut TraitItem,
}

impl<'a> TraitMember<'a> {
    fn from_item(item: &'a mut TraitItem) -> Option<Self> {
        if should_skip_member(item.attrs_mut()) {
            return None;
        }

        let (ident, kind) = match item {
            TraitItem::Const(item) => (item.ident.clone(), MemberKind::Constant),
            TraitItem::Fn(item) => (item.sig.ident.clone(), MemberKind::Function),
            TraitItem::Type(item) => (item.ident.clone(), MemberKind::Type),
            _ => return None,
        };

        Some(Self { ident, kind, item })
    }
}

#[derive(Debug)]
struct EnumMember<'a> {
    ident: Ident,
    kind: MemberKind,
    item: &'a mut Variant,
}

impl<'a> EnumMember<'a> {
    fn from_item(item: &'a mut Variant) -> Option<Self> {
        if should_skip_member(&item.attrs) {
            return None;
        }

        Some(Self {
            ident: item.ident.clone(),
            kind: MemberKind::Variant,
            item,
        })
    }
}

fn load_member_documentation(
    ident: &Ident,
    member_ident: &str,
    kind: MemberKind,
    attrs: &mut Vec<Attribute>,
    config: &AutoDocArgs,
    additional_paths: &mut Vec<String>,
) -> Result<(), Error> {
    let member_path = config
        .member_path
        .as_deref()
        .unwrap_or("docs/{type}/{member}.md")
        .replace("{type}", &ident.to_string())
        .replace("{member}", member_ident)
        .replace("{kind}", kind.as_str());
    let member_files = vec![member_path];
    let (member_doc, member_paths) = load_documentation(&member_files, ident.span())?;
    attrs.push(parse_quote!(#[doc = #member_doc]));
    additional_paths.extend(member_paths);
    Ok(())
}

fn should_skip_member(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr.parse_args::<Meta>().map_or(
                false,
                |meta| matches!(meta, Meta::Path(path) if path.is_ident("hidden")),
            )
    })
}

impl MemberKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Field => "field",
            Self::Function => "function",
            Self::Constant => "constant",
            Self::Type => "type",
            Self::Variant => "variant",
        }
    }
}

trait ItemAttrs {
    fn attrs_mut(&mut self) -> &mut Vec<Attribute>;
}

impl ItemAttrs for ImplItem {
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

impl ItemAttrs for TraitItem {
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        match self {
            TraitItem::Const(item) => &mut item.attrs,
            TraitItem::Fn(item) => &mut item.attrs,
            TraitItem::Type(item) => &mut item.attrs,
            TraitItem::Macro(item) => &mut item.attrs,
            _ => panic!("auto_doc: unsupported trait item"),
        }
    }
}
