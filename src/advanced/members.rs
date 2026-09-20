use super::AutoDocArgs;
use crate::common::{documentation_attribute, load_documentation};
use proc_macro2::{Ident, Span};
use syn::{Attribute, Error, Fields, ImplItem, Item, Meta, TraitItem, Variant};

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
) -> Result<(), Error> {
    match item {
        Item::Struct(item_struct) => {
            if let Fields::Named(fields) = &mut item_struct.fields {
                for member in &mut fields.named {
                    let Some(member_ident) = &member.ident else {
                        continue;
                    };

                    if should_skip_member(&member.attrs) {
                        continue;
                    }

                    load_member_documentation(
                        ident,
                        &member_ident.to_string(),
                        MemberKind::Field,
                        member_ident.span(),
                        &mut member.attrs,
                        config,
                    )?;
                }
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
                    member_info.ident.span(),
                    member_info.item.attrs_mut(),
                    config,
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
                    member_info.ident.span(),
                    member_info.item.attrs_mut(),
                    config,
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
                    member_info.ident.span(),
                    &mut member_info.item.attrs,
                    config,
                )?;

                if let Fields::Named(fields) = &mut member_info.item.fields {
                    for member in &mut fields.named {
                        let Some(member_ident) = &member.ident else {
                            continue;
                        };

                        if should_skip_member(&member.attrs) {
                            continue;
                        }

                        let member_name = format!("{}/{}", member_info.ident, member_ident);
                        load_member_documentation(
                            ident,
                            &member_name,
                            MemberKind::Field,
                            member_ident.span(),
                            &mut member.attrs,
                            config,
                        )?;
                    }
                }
            }
        }
        Item::Union(item_union) => {
            for member in &mut item_union.fields.named {
                let Some(member_ident) = &member.ident else {
                    continue;
                };

                if should_skip_member(&member.attrs) {
                    continue;
                }

                load_member_documentation(
                    ident,
                    &member_ident.to_string(),
                    MemberKind::Field,
                    member_ident.span(),
                    &mut member.attrs,
                    config,
                )?;
            }
        }
        _ => {
            return Err(Error::new(
                ident.span(),
                "auto_doc: `members` requires a struct, impl, trait, or enum",
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
        let (ident, kind) = match item {
            ImplItem::Const(item) => (item.ident.clone(), MemberKind::Constant),
            ImplItem::Fn(item) => (item.sig.ident.clone(), MemberKind::Function),
            ImplItem::Type(item) => (item.ident.clone(), MemberKind::Type),
            _ => return None,
        };

        if should_skip_member(item.attrs_mut()) {
            return None;
        }

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
        let (ident, kind) = match item {
            TraitItem::Const(item) => (item.ident.clone(), MemberKind::Constant),
            TraitItem::Fn(item) => (item.sig.ident.clone(), MemberKind::Function),
            TraitItem::Type(item) => (item.ident.clone(), MemberKind::Type),
            _ => return None,
        };

        if should_skip_member(item.attrs_mut()) {
            return None;
        }

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
    span: Span,
    attrs: &mut Vec<Attribute>,
    config: &AutoDocArgs,
) -> Result<(), Error> {
    let configured_member_path = config.member_path.as_deref();
    if config.source.is_none()
        && configured_member_path.is_some_and(|path| path.contains("{source}"))
    {
        return Err(Error::new(
            span,
            "auto_doc: `member_path` contains `{source}`, but `source` was not provided",
        ));
    }

    let member_path = config
        .member_path
        .as_deref()
        .or_else(|| {
            config
                .source
                .as_deref()
                .filter(|source| !source.is_empty())
                .map(|_| "{docs}/{source}/{type}/{member}.md")
        })
        .unwrap_or("{docs}/{type}/{member}.md")
        .replace("{source}", config.source.as_deref().unwrap_or(""))
        .replace("{docs}", "docs")
        .replace("{type}", &ident.to_string())
        .replace("{member}", member_ident)
        .replace("{kind}", kind.as_str());

    let member_files = vec![member_path];

    let member_contents = load_documentation(&member_files, ident.span())?.contents;

    attrs.extend(documentation_attribute(
        &member_files,
        &member_contents,
        span,
    ));

    Ok(())
}

fn should_skip_member(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr
                .parse_args::<Meta>()
                .is_ok_and(|meta| matches!(meta, Meta::Path(path) if path.is_ident("hidden")))
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
