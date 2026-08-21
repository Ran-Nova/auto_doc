# auto_doc 📖

A lightweight procedural attribute macro for embedding Markdown documentation into Rust items.

## What it does

`auto_doc` reads one or more Markdown files and injects their contents into the generated Rust documentation for the annotated item.

This is useful when you want to keep long docs outside the source file and still get proper Rust doc output.

## Installation

```toml
[dependencies]
auto_doc = "0.1.4"
```

## Usage

### Default behavior

If no path is provided, the macro looks for:

```text
docs/<ItemName>.md
```

```rust
use auto_doc::auto_doc;

#[auto_doc]
pub fn my_function() {}
```

### Single custom path

```rust
use auto_doc::auto_doc;

#[auto_doc(path = "docs/api.md")]
pub struct MyType;
```

### Multiple files

```rust
use auto_doc::auto_doc;

#[auto_doc("docs/intro.md", "docs/advanced.md")]
pub trait MyTrait {}
```

### Multiple named paths

```rust
use auto_doc::auto_doc;

#[auto_doc(paths = "docs/a.md", paths = "docs/b.md")]
pub fn complex_function() {}
```

## Supported item kinds

The macro supports item declarations such as:

- `struct`
- `enum`
- `trait`
- `fn`
- `const`
- `static`
- `type`
- `impl`

## Notes

- Paths are resolved relative to the crate root by default.
- Absolute paths are also accepted.
- The macro reads the Markdown files at compile time and embeds them into the generated doc text.

## Why use it

- keep documentation outside source files;
- easier to maintain long docs;
- works naturally with Rust documentation tooling.
