# auto_doc 📖

A lightweight, compiler-friendly procedural attribute macro that pulls external Markdown files directly into your Rust documentation.

Keep your source code clean and your documentation comprehensive.

## Why `auto_doc`?

Standard Rust allows using `#[doc = include_str!("...")]`, but it often breaks IDE hover tooltips in `rust-analyzer` or expands into messy compiler-internal artifacts (`builtin #include_bytes`) due to macro expansion quirks.

`auto_doc` reads the files directly during compilation and attaches a clean, monolithic string literal to your items, ensuring **perfect rendering** in both `cargo doc` and VS Code / IntelliJ Rust.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
auto_doc = "0.1.0"
```

## Usage

By default, if no path is provided, `auto_doc` looks for a file named `docs/<item_name>.md` relative to your workspace root.

## Basic Example

```rs
use auto_doc::auto_doc;

// Looks for "docs/my_function.md"
#[auto_doc]
pub fn my_function() {
    // ...
}
```

## Custom Paths & Multiple Files

You can specify explicit paths or even combine multiple Markdown files into a single documentation block:

```rs
// Explicit single path
#[auto_doc(path = "architecture/SAFETY.md")]
pub struct SecureVault;

// Multiple files combined in order
#[auto_doc("docs/intro.md", "docs/api_spec.md")]
pub trait CoreEngine {
    fn run(&self);
}

// Named multiple paths syntax
#[auto_doc(paths = "docs/A.md", paths = "docs/B.md")]
pub fn complex_operation() {}
```
