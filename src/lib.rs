//! # Features
//!
//! ## Default
//!
//! Lightweight mode without optional parser dependencies.
//!
//! ## `advanced`
//!
//! Enables `darling` and full `syn` AST support.
//! Provides `members = true` and `member_path` for documenting items inside `impl` blocks.

use proc_macro::TokenStream;

mod common;
#[cfg(any(not(feature = "advanced"), test))]
mod default;

#[cfg(feature = "advanced")]
mod advanced;

#[cfg(test)]
mod tests;

/// Automatically generates documentation for the given item based on the
/// provided attributes.
///
/// Supported forms:
/// - `#[auto_doc]` - use `docs/{item}.md`
/// - `#[auto_doc("docs/Item.md")]` - multiple paths work too
/// - `#[auto_doc(path = "docs/Item.md")]`
/// - `#[auto_doc(paths = ["docs/A.md", "docs/B.md"])]`
///
/// With the `advanced` feature:
/// - `#[auto_doc(members = true)]` documents associated functions, types, and constants in an `impl` block.
/// - `member_path = "docs/{type}/{member}.md"` customizes the member documentation path.
///
/// The `member_path` template supports `{type}` and `{member}` placeholders.
///
/// If no paths are provided, the macro falls back to `docs/<ItemName>.md`.
#[proc_macro_attribute]
pub fn auto_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_auto_doc(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

#[cfg(feature = "advanced")]
use advanced::impl_auto_doc;
#[cfg(not(feature = "advanced"))]
use default::impl_auto_doc;
