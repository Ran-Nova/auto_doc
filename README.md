# auto_doc 📖

A lightweight procedural attribute macro for embedding Markdown documentation into Rust items.

## What it does

`auto_doc` reads one or more Markdown files and injects their contents into the generated Rust documentation for the annotated item.

This is useful when you want to keep long docs outside the source file and still get proper Rust doc output.

## Installation

```toml
[dependencies]
auto_doc = "0.2.10"
```

*I recommend keeping **`auto_doc`** in your **`Cargo.toml`** updated to the latest version for stable library operation.*

## Features

### Default

The `default` feature has no optional parser dependencies. It supports:

* `#[auto_doc]`;
* `path = "..."`;
* repeated `paths = "..."` arguments; (`advanced` not supported it)
* positional paths such as `#[auto_doc("docs/api.md")]`.
* `source = "..."` - change your path to `docs/{source}/<ItemName>.md`

### `advanced`

The `advanced` feature enables `darling` for extensible attribute argument parsing and full `syn` AST support. Generic items and implementations such as `impl<T> ... for Type<T>` are handled through the advanced parser.

Enable it in `Cargo.toml`:

```toml
[dependencies]
auto_doc = { version = "0.2.10", features = ["advanced"] }
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

With the `advanced` feature enabled, use `members` to load documentation for fields and named items inside a `struct`, `impl`, `trait`, or `enum`. The equivalent `members = true` form remains supported. Member documentation uses the `docs/<Type>/<member>.md` path:

```rust
use auto_doc::auto_doc;

#[auto_doc(members)]
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

The member path can be customized with the `{source}`, `{docs}`, `{type}`, `{member}`, and
`{kind}` placeholders. The `{kind}` value is `field`, `function`, `constant`, `type`, or `variant`:

```rust
use auto_doc::auto_doc;

#[auto_doc(
	members,
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

#[auto_doc(members)]
struct MyType {
	#[doc(hidden)]
	internal_only: (),

	public_api: (),
}
```

In this case, `internal_only` is ignored by `auto_doc`, while `public_api` is still documented from its matching member file.

You can use `source = "..."` on `members` (or single path/s)

```rust
use auto_doc::auto_doc;

#[auto_doc(members, source = "api")]
trait MyTrait {
	pub fn get() {}
	pub fn set() {}
}
```

This example expects the following file:

```text
docs/api/MyTrait.md
docs/api/MyTrait/get.md
docs/api/MyTrait/set.md
```

## Supported item kinds

The macro supports item declarations such as:

* `struct`
* `enum`
* `trait`
* `fn`
* `const`
* `static`
* `type`
* `impl` (available in the `advanced` feature with `members`)

## Notes

* Paths are resolved relative to the crate root by default.
* Absolute paths are also accepted.
* The macro reads the Markdown files at compile time and embeds them into the generated doc text.
* Ignores other attribute blocks (due to procedural macro constraints, since v0.2.4).
* Since `0.2.8`, generated documentation with `# Errors`, `# Panics`, or `# Safety` suppresses the corresponding Clippy false positive, exclide for `async fn`.
* Since `0.2.9`, you can use `{source}` and `source = "folder/sub-folder`. Automatic path relative to file could not be implemented (but an attempt was made)
* Since `0.2.10`, if the generated documentation has `# Errors`, `# Panics`, or `# Safety`, the macro adds `#[allow(clippy::missing_errors_doc)]`, `#[allow(clippy::missing_panics_doc)]`, or `#[allow(clippy::missing_safety_doc)]` respectively. Clippy, stop, i have it tags.

## Why use it

* keep documentation outside source files;
* easier to maintain long docs;
* works naturally with Rust documentation tooling.
