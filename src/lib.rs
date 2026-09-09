//! # Features
//!
//! ## Default
//!
//! Lightweight mode without optional parser dependencies.
//!
//! ## `advanced`
//!
//! Enables `darling` and full `syn` AST support.
//! Provides `members` and `member_path` for documenting fields, variants, and inner items.
//! The advanced parser also supports generic items and implementations.

use proc_macro::TokenStream;

mod common;

#[cfg(feature = "advanced")]
mod advanced;
#[cfg(not(feature = "advanced"))]
mod default;

/// Automatically generates documentation for the given item based on the
/// provided attributes.
/// Generated documentation is attached to the item so rustdoc and Clippy can
/// process sections such as `# Errors`, `# Panics`, and `# Safety` normally.
///
/// Supported in both feature modes:
/// - `#[auto_doc]` - use `docs/{item}.md`
/// - `#[auto_doc("docs/Item.md")]` - positional paths
/// - `#[auto_doc(path = "docs/Item.md")]`
/// - `#[auto_doc(paths = ["docs/A.md", "docs/B.md"])]`
/// - `#[auto_doc(paths = "docs/A.md", paths = "docs/B.md")]` (only `default` feature)
///
/// Positional paths and the array form of `paths` are supported in both feature modes.
/// Repeating `paths = "..."` is supported only in the `default` feature.
///
/// With the `advanced` feature:
/// - `#[auto_doc(source = "api")]` uses `docs/api/{item}.md` as the default item path.
/// - `#[auto_doc(members)]` documents fields in `structs`, variants in `enums`, and members in `traits`/`impls`.
/// - `member_path = "docs/{type}/{member}.md"` customizes the member documentation path.
/// - `source = "folder/sub-folder"` prefixes the default item path and supplies `{source}` to `member_path`.
/// - `members` and `member_path` are valid only with the named-argument syntax.
///
/// The `member_path` template supports `{source}`, `{docs}`, `{type}`, `{member}`, and `{kind}` placeholders.
/// `{kind}` resolves to `field`, `function`, `constant`, `type`, or `variant` for each member.
/// `{source}` resolves to the explicit `source` argument and is supported in `member_path`.
/// `{docs}` resolves to the configured documentation root.
///
/// Members marked with `#[doc(hidden)]` are ignored when `members` is enabled.
///
/// If no paths are provided, the macro falls back to `docs/<ItemName>.md` for regular items.
/// `impl` blocks do not receive an item-level document because Rust does not apply `#[doc]`
/// attributes to them; their members can still be documented with `members`.
///
/// Named fields in `struct` and `enum` variants are processed as members.
/// Unnamed tuple fields are not processed separately.
#[proc_macro_attribute]
pub fn auto_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_auto_doc(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

#[cfg(feature = "advanced")]
use advanced::impl_auto_doc;
#[cfg(not(feature = "advanced"))]
use default::impl_auto_doc;
