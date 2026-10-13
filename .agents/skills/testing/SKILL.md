---
name: testing
description: How to design mosaico code so every component is testable in isolation, how to write and run tests, and which suites are safe to run on a developer machine. Use when designing a new component, before running any cargo test command, or when adding tests.
---

# Testing

## Build

```sh
cargo build                # all workspace crates
cargo build -p mosaico     # CLI binary only
cargo fmt --all            # format before committing
```

## Safe to run locally

These match CI and never touch the desktop:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --doc
cargo test -p mosaico --test cli -- help_exits_successfully version_exits_successfully debug_list_subcommand_runs --exact
```

Run a single unit test by name with `cargo test --lib <name>`.

## Not safe without asking

- `cargo test` and `cargo test --workspace` unfiltered.
- `cargo test -p mosaico --test cli` unfiltered. `start_and_stop_lifecycle` and `second_daemon_is_rejected`
  start real daemons that fight over the single-instance mutex; the second can block forever and leave an
  orphan daemon tiling the user's windows.
- `cargo test -p mosaico --test window_management`, `--test bar_reservation` and `--test display_change`. They
  start the daemon, launch notepad and tile the live desktop. CI runs them on a dedicated runner.
  `window_management` also overwrites the real `~/.config/mosaico/config.toml` while it runs (#61).
- `cargo test -p mosaico --test display_change -- --ignored` additionally disconnects and reconnects the real
  monitors with `DisplaySwitch`. Only run it with the user's go-ahead, on a machine in extend mode with two or more
  monitors.

If you must run one of these, tell the user first, run one suite at a time, and afterwards make sure no daemon
is left behind (`taskkill /IM mosaico.exe /F`). When asked to "run all the tests", run the safe set and say
which suites were skipped and why.

## Design for isolation

Every component must be testable on its own: no desktop, network, real config directory or running daemon.
Pass dependencies in instead of reaching for them.

| Dependency | Pass in as | Not |
| --- | --- | --- |
| OS (windows, monitors, hotkeys, processes) | a core trait, faked in tests | a concrete platform type or free function |
| Time | a `now: Instant` parameter or a clock trait | `Instant::now()` inside the logic |
| File system | a directory or path (config store, daemon instance) | `config_dir()` / `pid_path()` called internally |
| Network | an HTTP client trait | `http::get` called directly |
| Daemon / IPC | a client trait | `ipc::send_command` called directly |
| Exit code and output | a returned outcome and a writer | `std::process::exit` or `println!` in the logic |

Keep the side-effecting shell thin and the logic pure, shaped like `(state, input, now) -> (state, effects)`.
The shell gathers inputs, calls the logic and carries out the effects. Tests drive the logic directly and assert
on the effects.

Examples to copy:

- `wait_until_gone` in `crates/mosaico/src/commands/stop.rs` takes the liveness check and timings as parameters,
  so its tests never start a process.
- `DisplaySettle` in `crates/mosaico-windows/src/daemon_settle.rs` takes `now`, so settle timing is tested
  without sleeping.

Red flags:

- `Instant::now()` or `SystemTime::now()` inside decision logic
- `std::process::exit` outside `main.rs`
- public functions that locate their own files through `config_dir()` or a fixed name
- global names (pipe, mutex, file paths) as `const` with no way to override them
- a test that writes to the user's config directory, needs a running daemon to check logic, or sleeps to wait
  for something

When a test needs one of these, add the missing seam instead of working around it. Known gaps are tracked in
#57 and #61 to #64.

## Writing tests

- Unit tests go in the same file (`#[cfg(test)] mod tests { ... }`) or a sibling `tests` module.
- Integration tests live in `crates/<crate>/tests/`. New features need integration coverage.
- Test logic in `mosaico-core` against fake trait implementations so it runs on any OS. If a test needs a real
  window or monitor to exercise logic that does not call the OS, the logic is probably in the wrong crate (see
  [platform-boundary](../platform-boundary/SKILL.md)).
- Desktop tests capture screenshots with `scripts/screenshot.ps1 -Mode all` into `target/test-screenshots/`.
- Structure tests with Arrange / Act / Assert comments:

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
