# Mosaico — Development Plan

A tiling window manager for Windows, written in Rust. Inspired by komorebi.

---

## Phase 1: Project Initialization

**Goal:** Have a building Rust executable and a git repo ready to push.

**Status:** Complete.

### Tasks

- [x] Scaffold project with `cargo init`
- [x] Update `Cargo.toml` metadata (description, authors, license, repository)
- [x] Verify `cargo build` and `cargo run` produce a working binary
- [x] Add a proper `.gitignore` for Rust (ignore `/target`, etc.)
- [x] `git init` + initial commit
- [ ] Create remote repo on GitHub (manual — user handles this)
- [ ] Push initial commit

### Notes

- Edition is set to `2024` (Rust 2024 edition — requires Rust 1.85+)
- Binary name: `mosaico`

---

## Phase 2: CLI with clap

**Goal:** Introduce a POSIX-style command-line interface using the `clap` crate so
Mosaico can be invoked with subcommands like `mosaico start`, `mosaico stop`, etc.

### Why clap?

`clap` is the Rust equivalent of .NET's `System.CommandLine`. It is the most widely
used CLI parsing library in the Rust ecosystem. Key features:

- Derive macros for declarative argument definitions (similar to attributes in C#)
- Auto-generated `--help` and `--version`
- Subcommand support
- Argument validation, default values, environment variable bindings
- Shell completions generation

### Tasks

- [x] Add `clap` dependency with the `derive` feature to `Cargo.toml`
- [x] Define a `Cli` struct using `#[derive(Parser)]`
- [x] Add initial subcommands (stub implementations that print a message):
  - `start` — Start the window manager
  - `stop` — Stop a running instance
  - `status` — Show whether Mosaico is currently running
- [x] `--version` and `--help` come for free via clap
- [x] Write one integration test: verify the binary runs and `--help` exits 0

### Example code sketch

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mosaico", version, about = "A tiling window manager for Windows")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the window manager
    Start,
    /// Stop the window manager
    Stop,
    /// Show current status
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => println!("Starting Mosaico..."),
        Commands::Stop => println!("Stopping Mosaico..."),
        Commands::Status => println!("Mosaico status: not implemented yet"),
    }
}
```

### Rust learning notes

- `#[derive(...)]` — procedural macros that generate trait implementations at
  compile time. Similar to C# source generators or attributes.
- `enum` with variants — Rust's enums are "tagged unions" / discriminated unions.
  Each variant can hold different data. `Commands::Start` is like a sealed class
  hierarchy in C#.
- `match` — exhaustive pattern matching. The compiler forces you to handle every
  variant, so you can't forget a case.

---

## Future Phases (not yet planned in detail)

3. **Window enumeration** — Use the `windows-rs` crate to list open windows
4. **Window positioning** — Move/resize windows using Win32 API
5. **Tiling layouts** — Implement BSP or master-stack layout algorithms
6. **Hotkey system** — Global hotkey registration for keyboard-driven control
7. **Configuration** — TOML/YAML config file for user preferences
8. **Multi-monitor support**
9. **Window rules** — Per-application tiling behavior
10. **IPC** — Named pipe or socket for `mosaico start`/`stop` to communicate with
    a running daemon
