---
name: testing
description: How to write and run mosaico tests, and which suites are safe to run on a developer machine. Use before running any cargo test command or when adding tests.
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
- `cargo test -p mosaico --test display_change -- --ignored` additionally disconnects and reconnects the real
  monitors with `DisplaySwitch`. Only run it with the user's go-ahead, on a machine in extend mode with two or more
  monitors.

If you must run one of these, tell the user first, run one suite at a time, and afterwards make sure no daemon
is left behind (`taskkill /IM mosaico.exe /F`). When asked to "run all the tests", run the safe set and say
which suites were skipped and why.

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
