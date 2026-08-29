# auto_doc 📖

A lightweight procedural attribute macro for embedding Markdown documentation into Rust items.

## What it does

`auto_doc` reads one or more Markdown files and injects their contents into the generated Rust documentation for the annotated item.

This is useful when you want to keep long docs outside the source file and still get proper Rust doc output.

## Installation

```toml
[dependencies]
auto_doc = "0.2.4"
```

*I recommend updating **`auto_doc`** in your **`Cargo.toml`** to the **latest** version for stable library operation.*

## Features

### Default

The default mode has no optional parser dependencies. It supports:

- `#[auto_doc]`;
- `path = "..."`;
- repeated `paths = "..."` arguments;
- positional paths such as `#[auto_doc("docs/api.md")]`.

### `advanced`

The `advanced` feature enables `darling` for extensible attribute argument parsing and full `syn` AST support. Generic items and implementations such as `impl<T> ... for Type<T>` are handled through the advanced parser.

Enable it in `Cargo.toml`:

```toml
[dependencies]
auto_doc = { version = "0.2.3", features = ["advanced"] }
```

The positional path syntax remains available in this mode.

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
pub type MyType;
```

### Single file

```rust
use auto_doc::auto_doc;
#[auto_doc("docs/struct.md")]
pub struct MyStruct;
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

#[auto_doc(paths = ["docs/a.md", "docs/b.md"])]
pub fn complex_function() {}
```

### Documenting impl members

With the `advanced` feature enabled, use `members = true` to load documentation for named items inside an `impl` block. The implementation documentation uses the normal `docs/<Type>.md` path, while member documentation uses `docs/<Type>/<member>.md`:

```rust
use auto_doc::auto_doc;

#[auto_doc(members = true)]
impl<T> MyType<T> {
	pub fn value(&self) {}
}
```

This example expects the following files:

```text
docs/MyType.md
docs/MyType/value.md
```

The option applies to associated functions, types, and constants. It must be used on an `impl` block and is available only in `advanced` mode.

The member path can be customized with the `{type}`, `{member}`, and `{kind}` placeholders. The `{kind}` value is `function`, `constant`, or `type`:

```rust
use auto_doc::auto_doc;

#[auto_doc(
	members = true,
	member_path = "reference/{type}/{kind}/{member}.md"
)]
impl<T> MyType<T> {
	pub fn value(&self) {}
}
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
- Ignores #[] blocks (this is because of proc_macro_(derive/attribute)) (0.2.4 or later)

## Why use it

- keep documentation outside source files;
- easier to maintain long docs;
- works naturally with Rust documentation tooling.
