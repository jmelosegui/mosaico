# Phase 2: CLI with clap

**Status:** Complete

**Goal:** Introduce a POSIX-style command-line interface using the `clap` crate so
Mosaico can be invoked with subcommands like `mosaico start`, `mosaico stop`, etc.

## Why clap?

`clap` is the Rust equivalent of .NET's `System.CommandLine`. It is the most widely
used CLI parsing library in the Rust ecosystem. Key features:

- Derive macros for declarative argument definitions (similar to attributes in C#)
- Auto-generated `--help` and `--version`
- Subcommand support
- Argument validation, default values, environment variable bindings
- Shell completions generation

## Tasks

- [x] Add `clap` dependency with the `derive` feature to `Cargo.toml`
- [x] Define a `Cli` struct using `#[derive(Parser)]`
- [x] Add initial subcommands (stub implementations that print a message):
  - `start` — Start the window manager
  - `stop` — Stop a running instance
  - `status` — Show whether Mosaico is currently running
- [x] `--version` and `--help` come for free via clap
- [x] Write integration tests (help, version, start subcommand)

## Rust learning notes

- `#[derive(...)]` — procedural macros that generate trait implementations at
  compile time. Similar to C# source generators or attributes.
- `enum` with variants — Rust's enums are "tagged unions" / discriminated unions.
  Each variant can hold different data. `Commands::Start` is like a sealed class
  hierarchy in C#.
- `match` — exhaustive pattern matching. The compiler forces you to handle every
  variant, so you can't forget a case.
