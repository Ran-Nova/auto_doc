# auto_doc 📖

A lightweight procedural attribute macro for embedding Markdown documentation into Rust items.

## What it does

`auto_doc` reads one or more Markdown files and injects their contents into the generated Rust documentation for the annotated item.

This is useful when you want to keep long docs outside the source file and still get proper Rust doc output.

## Installation

```toml
[dependencies]
auto_doc = "0.2.7"
```

*I recommend keeping **`auto_doc`** in your **`Cargo.toml`** updated to the latest version for stable library operation.*

## Features

### Default

The `default` feature has no optional parser dependencies. It supports:

* `#[auto_doc]`;
* `path = "..."`;
* repeated `paths = "..."` arguments; (`advanced` not supported it)
* positional paths such as `#[auto_doc("docs/api.md")]`.

### `advanced`

The `advanced` feature enables `darling` for extensible attribute argument parsing and full `syn` AST support. Generic items and implementations such as `impl<T> ... for Type<T>` are handled through the advanced parser.

Enable it in `Cargo.toml`:

```toml
[dependencies]
auto_doc = { version = "0.2.7", features = ["advanced"] }
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

> Note: The singular `path = "..."` argument cannot be repeated. To specify multiple files, use the `paths` syntax instead.

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

> Note: The `default` feature supports both repeated `paths = "..."` syntax and array syntax (`paths = [...]`), giving you full flexibility.

### Documenting members

With the `advanced` feature enabled, use `members = true` to load documentation for fields and named items inside a `struct`, `impl`, `trait`, or `enum`. Member documentation uses the `docs/<Type>/<member>.md` path:

```rust
use auto_doc::auto_doc;

#[auto_doc(members = true)]
impl<T> MyType<T> {
	pub fn value(&self) {}
}
```

This example expects the following file:

```text
docs/MyType/value.md
```

For traits and enums, the main item documentation is loaded from `docs/<Type>.md` as usual. `docs/MyType.md` is ignored for impl blocks because `#[doc]` attributes have no effect on them.

The option applies to fields in structs, associated functions, types, and constants in impls and traits, and to variants in enums. Tuple struct fields use their numeric index as `{member}`. It is available only in `advanced` mode.

The member path can be customized with the `{type}`, `{member}`, and `{kind}` placeholders. The `{kind}` value is `field`, `function`, `constant`, `type`, or `variant`:

```rust
use auto_doc::auto_doc;

#[auto_doc(
	members = true,
	member_path = "reference/{type}/{kind}/{member}.md"
)]
enum MyType {
	Name,
	Age,
	Email
}
```

> Note: Tuple fields in `struct` declarations are also processed and use their numeric index as `{member}`. Tuple or struct fields inside an `enum` variant are not processed separately; only the variant itself is documented.

You can skip individual members from being documented by marking them with `#[doc(hidden)]`:

```rust
use auto_doc::auto_doc;

#[auto_doc(members = true)]
struct MyType {
	#[doc(hidden)]
	internal_only: (),

	public_api: (),
}
```

In this case, `internal_only` is ignored by `auto_doc`, while `public_api` is still documented from its matching member file.

## Supported item kinds

The macro supports item declarations such as:

* `struct`
* `enum`
* `trait`
* `fn`
* `const`
* `static`
* `type`
* `impl` (available in the `advanced` feature with `members = true`)

## Notes

* Paths are resolved relative to the crate root by default.
* Absolute paths are also accepted.
* The macro reads the Markdown files at compile time and embeds them into the generated doc text.
* Ignores other attribute blocks (due to procedural macro constraints, since v0.2.4).

## Why use it

* keep documentation outside source files;
* easier to maintain long docs;
* works naturally with Rust documentation tooling.
