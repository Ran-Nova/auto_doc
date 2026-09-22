# auto_doc 📖

A lightweight procedural attribute macro for embedding Markdown documentation into Rust items.

## What it does

`auto_doc` reads one or more Markdown files and injects their contents into the generated Rust documentation for the annotated item.

This is useful when you want to keep long docs outside the source file and still get proper Rust doc output.

## Installation

```toml
[dependencies]
auto_doc = "0.2.15"
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

In addition to the default syntax, this feature supports `source = "..."`,
`members`, and `member_path`.

Enable it in `Cargo.toml`:

```toml
[dependencies]
auto_doc = { version = "0.2.15", features = ["advanced"] }
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
>
> If you use `advanced` feature - you can use `source = "api"` for find `MyType.md` in `docs/api`

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

With the `advanced` feature enabled, use `members` to load documentation for fields and named items inside a `struct`, `impl`, `trait`, `union` or `enum`. The equivalent `members = true` form remains supported. Member documentation uses the `docs/<Type>/<member>.md` path:

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

The option applies to named fields in structs and enum variants, associated functions, types, and constants in impls and traits, and to variants in enums. Unnamed tuple fields are ignored. It is available only in `advanced` mode.

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

> Note: Unnamed tuple fields in `struct` declarations and enum variants are not processed separately. Named fields in enum variants use `{member}` values such as `First/value`.

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

You can use `source = "..."` with `members` (`advanced` feature only):

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
* `union`
* `macro_rules`

## Notes

* Paths are resolved relative to the crate root by default.
* Absolute paths are also accepted.
* The macro reads the Markdown files at compile time and embeds them into the generated doc text.
* Ignores other attribute blocks (due to procedural macro constraints, since `0.2.4`)
* Since `0.2.9`, the `advanced` feature supports `{source}` and `source = "folder/sub-folder"`. Automatic paths relative to the source file are not supported. (But there was an attempt to implement it. Config too)
* Since `0.2.11`, generated documentation uses `///`-equivalent `#[doc]` attributes instead of block comments. Clippy recognizes generated `# Errors`, `# Panics`, and `# Safety` sections without automatic lint suppressions. The problem was with the missing Span, which was fixed in this version.
* Since `0.2.12`, named fields in standard structs and enum variants are documented as members. Unnamed fields in tuple structs and tuple enum variants are ignored (changed only docs on `0.2.12`-`0.2.13`). (`0.2.13`) There is no such thing as a named tuple struct — I forgot about that, sorry xD
* Since `0.2.14` added support `macro_rules` and `union` on both features. `members` work on `union` too.
* Version `0.2.15` (`advanced` feature) have only error text changes (i forgot add `union` in error)

## Roadmap

A separate companion crate is planned to support the broader `advanced` workflow, especially when configuration and documentation are spread across a large number of Markdown files and directories. The `members` workflow is one of the problems it is intended to address. The crate currently exists only as a design on paper; its name and implementation will be announced later.

I’m thinking about the `args` argument—it would be like `members`, but for the arguments in a `fn` (possibly even for unnamed `fn`s like `||`), and it should work with `members`. However, that won't happen before version 0.3, since right now it would just add more `<Type>/<Member><ArgsName>.md` files to the docs ;D

The companion crate is planned to be released together with `auto_doc` `0.3.0`. Until then, this release is `auto_doc` `0.2.15`.

## Why use it

* keep documentation outside source files;
* easier to maintain long docs;
* works naturally with Rust documentation tooling.
