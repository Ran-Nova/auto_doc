use proc_macro2::{Ident, Span};
use syn::{Error, Item, Type};

pub(crate) struct AdvancedItem {
    pub(crate) ident: Ident,
    pub(crate) is_impl: bool,
}

pub(crate) fn advanced_item_ident(item: &Item) -> Result<AdvancedItem, Error> {
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
        Item::Impl(item) => {
            let self_ty = &*item.self_ty;
            let ident = match self_ty {
                Type::Path(type_path) => type_path
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident.clone())
                    .ok_or_else(|| {
                        Error::new(Span::call_site(), "auto_doc: unsupported impl target")
                    })?,
                Type::Reference(reference) => match &*reference.elem {
                    Type::Path(type_path) => type_path
                        .path
                        .segments
                        .last()
                        .map(|segment| segment.ident.clone())
                        .ok_or_else(|| {
                            Error::new(Span::call_site(), "auto_doc: unsupported impl target")
                        })?,
                    _ => {
                        return Err(Error::new(
                            Span::call_site(),
                            "auto_doc: unsupported impl target",
                        ))
                    }
                },
                Type::Tuple(_) => {
                    return Err(Error::new(
                        Span::call_site(),
                        "auto_doc: unsupported impl target",
                    ))
                }
                _ => {
                    return Err(Error::new(
                        Span::call_site(),
                        "auto_doc: unsupported impl target",
                    ))
                }
            };

            Ok(AdvancedItem {
                ident,
                is_impl: true,
            })
        }
        _ => Err(Error::new(
            Span::call_site(),
            "auto_doc: unsupported item type",
        )),
    }
}
