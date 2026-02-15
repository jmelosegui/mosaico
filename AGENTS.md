# Agent Guidelines for Mosaico

This file contains instructions for AI coding agents working in this repository.

## Project Overview

Mosaico is a cross-platform tiling window manager written in Rust. The project
is structured as a Cargo workspace with multiple crates under `crates/`.

## Build / Lint / Test Commands

```sh
cargo build                          # Build all workspace crates
cargo build -p mosaico               # Build only the CLI binary
cargo test                           # Run all tests across the workspace
cargo test -p mosaico                # Run tests for the CLI crate only
cargo test help_exits_successfully   # Run a single test by name
cargo clippy --workspace             # Run linter across all crates
cargo fmt --all                      # Format all code
cargo fmt --all -- --check           # Check formatting without modifying
```

## Workspace Structure

```
Cargo.toml                  # Workspace root
crates/
  mosaico/                  # CLI binary
  mosaico-core/             # Platform-agnostic traits and types
  mosaico-windows/          # Windows platform implementation
```

- `mosaico` depends on `mosaico-core` and `mosaico-windows`
- `mosaico-windows` depends on `mosaico-core`
- Platform-specific code lives in its own crate, never in `mosaico-core`
- `mosaico-core` defines traits; platform crates implement them

## Rust Coding Conventions

Follow the official Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

### Naming

- Types and traits: `PascalCase`
- Functions, methods, variables: `snake_case`
- Constants and statics: `SCREAMING_SNAKE_CASE`
- Crate names: `kebab-case` in Cargo.toml, `snake_case` when imported
- Modules: `snake_case`
- Enum variants: `PascalCase`
- No `I` prefix on traits (use `Window` not `IWindow`)
- Getters: `field()` not `get_field()`
- Setters: `set_field()` not `set_value()`
- Conversions: `to_x()` (expensive), `as_x()` (cheap/borrow), `into_x()` (consuming)

### Formatting

- Run `cargo fmt --all` before committing
- Run `cargo clippy --workspace` and fix all warnings before committing
- Use `rustfmt` defaults — do not override `rustfmt.toml` settings

### File Organization

- No source file should exceed ~150 lines
- One concern per file
- Each CLI subcommand gets its own file under `commands/`
- Use `mod.rs` files to declare and re-export submodules
- Use `pub use` re-exports in `lib.rs` so consumers can use short paths

### Error Handling

- Never use `.unwrap()` or `.expect()` in library code (`mosaico-core`, `mosaico-windows`)
- `.expect("reason")` is acceptable in `main.rs` or CLI command handlers only
- Use `Result<T, E>` for fallible operations
- Propagate errors with the `?` operator

### Types and Traits

- Prefer `&str` over `String` in function parameters when possible
- Prefer returning `String` from trait methods when the caller needs ownership
- Derive common traits when appropriate: `Debug`, `Clone`, `PartialEq`
- Use `#[derive(...)]` rather than manual trait implementations unless custom
  behavior is needed

### Unsafe Code

- Keep `unsafe` blocks as small as possible
- Wrap every `unsafe` block in a safe function with a clear name
- Document why the `unsafe` block is sound with a `// SAFETY:` comment
- All `unsafe` code should live in platform crates, never in `mosaico-core`

### Documentation

- All public types, traits, and functions must have doc comments (`///`)
- Use `//` for implementation comments, not `/* */`
- Doc comments should describe *what* and *why*, not *how*

### Tests

- Integration tests live in `crates/<crate>/tests/`
- Unit tests go in the same file using `#[cfg(test)] mod tests { ... }`
- Use Arrange / Act / Assert comments to structure tests:
  ```rust
  #[test]
  fn it_works() {
      // Arrange
      let input = 42;

      // Act
      let result = do_something(input);

      // Assert
      assert_eq!(result, expected);
  }
  ```

### Imports

- Group imports in this order, separated by blank lines:
  1. Standard library (`std::`)
  2. External crates (`clap::`, `windows::`, etc.)
  3. Workspace crates (`mosaico_core::`, `mosaico_windows::`)
  4. Local modules (`crate::`, `super::`)
- Use specific imports, avoid glob imports (`use module::*`)

### Conditional Compilation

- Use `#[cfg(target_os = "windows")]` for platform-specific code
- Platform-specific logic belongs in its own crate, not behind `cfg` in shared code

### Git Commits

- Follow conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- Commit messages should explain *why*, not just *what*
- All commits must be GPG-signed
- Do not commit with `--no-gpg-sign`
- Do not commit unless explicitly asked by the user
- Update plan checkboxes in the same commit that completes the work

## Development Plan

See `.plans/plan.md` for the project roadmap and phase details.
