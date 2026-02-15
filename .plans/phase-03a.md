# Phase 3a: Restructure into Cargo Workspace

**Status:** Complete

**Goal:** Convert the project from a single crate into a multi-crate workspace,
establishing the foundation for cross-platform support.

## Why a workspace?

Rust workspaces allow multiple crates in one repo. Each crate compiles
independently, but they share a single `Cargo.lock` and build cache. This is how
most cross-platform Rust projects (ripgrep, alacritty) are structured.

On any given OS, `cargo build` only compiles the relevant platform crate — no need
for separate repos.

## Target structure

```
mosaico/
  Cargo.toml                  # workspace root
  crates/
    mosaico/                  # the CLI binary
      Cargo.toml
      src/
        main.rs
        commands/
          mod.rs
          start.rs
          stop.rs
          status.rs
    mosaico-core/             # platform-agnostic traits and types
      Cargo.toml
      src/
        lib.rs
        window.rs             # Window trait
        rect.rs               # Rect struct
    mosaico-windows/          # Windows platform implementation
      Cargo.toml
      src/
        lib.rs
```

## Tasks

- [x] Create workspace `Cargo.toml` at the root
- [x] Move existing CLI code into `crates/mosaico/`
- [x] Create `mosaico-core` crate with a `Window` trait and `Rect` struct
- [x] Create `mosaico-windows` crate (empty stubs, depends on `mosaico-core`)
- [x] Wire up dependencies between crates
- [x] Move integration tests into `crates/mosaico/tests/`
- [x] Verify `cargo build` and `cargo test` still pass
- [x] Commit

## Dependency graph

```
mosaico (binary)
  ├── mosaico-core
  └── mosaico-windows
        └── mosaico-core
```

## Rust learning notes

- `workspace` — a Cargo feature that groups multiple crates under one repo. Each
  crate has its own `Cargo.toml` but the root `Cargo.toml` lists all members.
- `path` dependencies — workspace crates reference each other by path
  (e.g. `mosaico-core = { path = "../mosaico-core" }`), not by version from
  crates.io.
- `pub use` / re-exports — `mosaico-core/src/lib.rs` can re-export types from
  submodules so consumers write `use mosaico_core::Window` instead of
  `use mosaico_core::window::Window`.
