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
cargo test -p mosaico --bins
cargo test --doc
cargo test -p mosaico-core --test '*'
cargo xtask metrics
cargo test -p mosaico --test cli -- help_exits_successfully version_exits_successfully debug_list_subcommand_runs \
  an_invalid_instance_name_is_refused a_test_instance_does_not_see_other_daemons \
  doctor_and_debug_list_show_monitors_as_the_daemon_sees_them --exact
```

`doctor_and_debug_list_show_monitors_as_the_daemon_sees_them` runs `mosaico doctor`, which only reads in a fresh test
instance (no config to migrate, no PID file to clean up); the test checks that it changes no file there.

`mosaico` is a binary crate, so `--lib` skips it. Its command unit tests (in `src/main.rs` and `commands/`) only
run with `--bins`. The `mosaico-core` integration tests in `crates/mosaico-core/tests/` each get a process of
their own, for state that can only be set once per process, such as the logger.

Run a single unit test by name with `cargo test --lib <name>`, or `cargo test -p mosaico --bins <name>` for a
CLI command test.

## Not safe without asking

- `cargo test` and `cargo test --workspace` unfiltered.
- `cargo test -p mosaico --test cli` unfiltered. `start_and_stop_lifecycle` and `second_daemon_is_rejected`
  start real daemons that tile the live desktop.
- `cargo test -p mosaico --test window_management`, `--test window_transitions`, `--test drag_and_drop`,
  `--test bar_reservation` and `--test display_change`. They start the daemon, launch notepad and tile the live
  desktop, and `drag_and_drop` presses, moves and releases the real pointer. CI runs them on a dedicated runner.
  `cargo test -p mosaico --test window_transitions --no-run` only builds that suite.

Every integration test runs its own daemon instance (`tests/common/mod.rs`): its own pipe, lock, PID file,
config and logs in a temporary directory, so none of them talks to the user's daemon or touches
`~/.config/mosaico`. A test that starts a daemon still puts a second window manager on the desktop, so it
refuses to run while the user's own daemon is running; ask the user to stop it first. New integration tests
must use `TestInstance` and never run `mosaico` without it.
- `cargo test -p mosaico --test display_change -- --ignored` additionally disconnects and reconnects the real
  monitors with `DisplaySwitch`. Only run it with the user's go-ahead, on a machine in extend mode with two or more
  monitors.

If you must run one of these, tell the user first, run one suite at a time, and afterwards make sure no daemon
is left behind (`taskkill /IM mosaico.exe /F`). When asked to "run all the tests", run the safe set and say
which suites were skipped and why.

## Metrics

`metrics/` holds two committed baselines, and `cargo xtask metrics` (run in CI) fails when a count differs from
them, in either direction:

- `metrics/tiler.toml`: what the tiling manager asks of the platform in common situations (opening a window,
  a burst of moves, a workspace switch, a display change), by kind of call, plus borders drawn from scratch.
  The scenarios are in `crates/mosaico-core/src/tiler/tests/metrics_scenarios.rs` and run on `FakePlatform`,
  after coalescing and echo handling. Fewer is better. Those named `_animated` (a move, a window opening next to
  three, one of four closing, a layout cycle, and monocle on and off) run with transitions on, as users get them
  by default, and count every call until the windows have settled, frames included. All the others run with
  transitions off (`manager_support::still()`), so they measure what the tiler asks of the platform without them.
- `metrics/code.toml`: per crate, tests (more is better), files and lines over the 150 line limit, `unsafe`
  keywords, `.unwrap()`/`.expect(` outside tests in the library crates, and OS names in `mosaico-core` (fewer is
  better).

When a count gets worse, fix the code. When a change is intended (a scenario added, a count improved, tests
added), record it with `cargo xtask metrics --bless` and commit the baseline with the change, so the review shows
it. An improvement fails until it is recorded, so it cannot slip back unnoticed. Add a scenario for any new kind of
work the tiler does.

`cargo xtask size` builds the release binary as the release workflow does and fails when it is over 2,000,000
bytes (`MAX_BYTES` in `xtask/src/size.rs`). It is a ceiling, not a baseline, because the size changes with every
Rust toolchain. It takes a release build, so CI runs it rather than every local check. When it fails, find what grew
with `cargo bloat --release -p mosaico --crates`.

## Design for isolation

Every component must be testable on its own: no desktop, network, real config directory or running daemon.
Pass dependencies in instead of reaching for them.

| Dependency | Pass in as | Not |
| --- | --- | --- |
| OS (windows, monitors, hotkeys, processes) | a core trait, faked in tests | a concrete platform type or free function |
| Time | a `now: Instant` parameter or a clock trait | `Instant::now()` inside the logic |
| File system | a directory or path (config store, daemon instance) | `config_dir()` / `pid_path()` called internally |
| Network | an HTTP client trait | `http::get` called directly |
| Daemon / IPC | `mosaico_core::ipc::IpcClient` / `IpcServer` | a concrete pipe or socket type |
| Exit code and output | a returned outcome and a writer | `std::process::exit` or `println!` in the logic |

Keep the side-effecting shell thin and the logic pure, shaped like `(state, input, now) -> (state, effects)`.
The shell gathers inputs, calls the logic and carries out the effects. Tests drive the logic directly and assert
on the effects.

Examples to copy:

- CLI commands take a `Context` (`crates/mosaico/src/commands/context.rs`) and return an `Exit`. Their tests
  run them against the fake `World` in `commands/test_support.rs`, so `stop` against a daemon that never
  exits, or `start` against one that is rejected at once, run in microseconds with no process involved.
- `Scheduler` in `crates/mosaico-core/src/daemon/scheduler.rs` takes `now`, so the daemon loop's ordering,
  display settling and flushing are tested without sleeping.
- `serve` in `crates/mosaico-core/src/ipc/transport.rs` is tested against in-memory connections, so the
  daemon's request handling needs no pipe.
- `ConfigStore` in `crates/mosaico-core/src/config/store/` is given its directory, so loading, merging and
  migrating the config files are tested on a temporary one that the test deletes when it ends.

Red flags:

- `Instant::now()` or `SystemTime::now()` inside decision logic
- `std::process::exit` outside `main.rs`
- public functions that locate their own files through `config_dir()` or a fixed name
- global names (pipe, mutex, file paths) as `const` with no way to override them
- a test that writes to the user's config directory, needs a running daemon to check logic, or sleeps to wait
  for something

When a test needs one of these, add the missing seam instead of working around it.

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
