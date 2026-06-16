# auto_doc 📖

A lightweight, compiler-friendly procedural attribute macro for pulling external Markdown files into Rust documentation.

Keep your source code clean while making your docs easier to author and maintain.

## Why use `auto_doc`?

Rust already supports `#[doc = include_str!("...")]`, but this can sometimes interfere with IDE hover tooltips in `rust-analyzer` and is harder to reason about once procedural macros are involved.

`auto_doc` reads Markdown files during compilation and emits a clean string literal for the final documentation item, giving you:

- consistent rendering in `cargo doc`
- working hover docs in supported editors such as VS Code
- simpler source code without inline markdown clutter

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
auto_doc = "0.1.0"
```

## Usage

By default, `auto_doc` looks for a file named `docs/<item_name>.md` relative to the workspace root where the item is defined.

### Basic example

```rust
use auto_doc::auto_doc;

// Loads documentation from "docs/my_function.md"
#[auto_doc]
pub fn my_function() {
    // ...
}
```

### Custom file paths

Specify one or more explicit Markdown file paths:

```rust
use auto_doc::auto_doc;

#[auto_doc(path = "architecture/SAFETY.md")]
pub struct SecureVault;
```

### Multiple files

Combine several Markdown files into one documentation block in declaration order:

```rust
use auto_doc::auto_doc;

#[auto_doc("docs/intro.md", "docs/api_spec.md")]
pub trait CoreEngine {
    fn run(&self);
}
```

You can also use repeated `paths =` attributes for the same effect:

```rust
use auto_doc::auto_doc;

#[auto_doc(paths = "docs/A.md", paths = "docs/B.md")]
pub fn complex_operation() {}
```

## Notes

- Paths are resolved relative to the crate root.
- Use Markdown files to keep long documentation outside of your source code.
- `auto_doc` is designed to preserve IDE documentation support and avoid macro expansion artifacts.
