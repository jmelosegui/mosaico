---
name: rust-conventions
description: Rust style and structure rules for mosaico. Use when writing or reviewing any Rust code in the workspace.
---

# Rust conventions

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

## Naming

- Types and traits: `PascalCase`. No `I` prefix on traits (`Window`, not `IWindow`).
- Functions, methods, variables, modules: `snake_case`.
- Constants and statics: `SCREAMING_SNAKE_CASE`.
- Crates: `kebab-case` in `Cargo.toml`, `snake_case` when imported.
- Getters are `field()`, not `get_field()`. Setters are `set_field()`.
- Conversions: `to_x()` (expensive), `as_x()` (cheap/borrow), `into_x()` (consuming).

## Formatting

- Run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` and fix everything before committing.
- Do not change `rustfmt.toml`.

## File organization

- Keep source files under about 150 lines, one concern per file.
- Each CLI subcommand gets its own file under `commands/`.
- Use `mod.rs` to declare and re-export submodules, and `pub use` in `lib.rs` for short paths.

## Error handling

- No `.unwrap()` or `.expect()` in library code (`mosaico-core`, `mosaico-windows`).
- `.expect("reason")` is acceptable in `main.rs` and CLI command handlers only.
- Return `Result<T, E>` for fallible operations and propagate with `?`.

## Types and traits

- Prefer `&str` over `String` in parameters; return `String` from trait methods when the caller needs ownership.
- Derive `Debug`, `Clone`, `PartialEq` where appropriate; implement traits by hand only for custom behaviour.

## Unsafe code

- Keep `unsafe` blocks as small as possible and wrap each one in a safe function with a clear name.
- Justify every block with a `// SAFETY:` comment.
- `unsafe` lives in platform crates only, never in `mosaico-core`.

## Documentation

- Every public type, trait and function has a `///` doc comment describing what and why, not how.
- Use `//` for implementation comments, not `/* */`.
- Trait docs in core state the contract platforms must honour. See the
  [platform-boundary](../platform-boundary/SKILL.md) skill.

## Imports

Group imports in this order, separated by blank lines:

1. Standard library (`std::`)
2. External crates (`clap::`, `windows::`, ...)
3. Workspace crates (`mosaico_core::`, `mosaico_windows::`)
4. Local modules (`crate::`, `super::`)

Use specific imports, not globs.

## Conditional compilation

- Use `#[cfg(target_os = "windows")]` only where the platform crate is selected (the CLI's platform module,
  `Cargo.toml` target dependencies).
- Platform-specific logic belongs in its own crate, not behind `cfg` in shared code.
