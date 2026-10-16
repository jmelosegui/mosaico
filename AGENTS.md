# Agent instructions

Mosaico is a cross-platform tiling window manager written in Rust. It is a Cargo workspace:

| Crate | Role |
| --- | --- |
| `crates/mosaico-core` | OS-agnostic traits, types and all decision logic (layouts, workspaces, config, matching) |
| `crates/mosaico-windows` | Windows implementation of the core traits (Win32, DWM, WinHTTP, registry) |
| `crates/mosaico` | CLI binary; depends on both |
| `crates/mosaico-metrics` | Metric reports and the committed baselines they are checked against (dev only) |
| `xtask` | Development tasks, run with `cargo xtask` (dev only) |

Only Windows ships today, but `mosaico-core` must stay portable so a `mosaico-linux` or `mosaico-macos` crate
can be added by implementing traits, not by copying logic. Every feature that touches the OS needs a contract
in core and an implementation in the platform crate. Missing contracts are the most common design defect in
this repo; see [platform-boundary](.agents/skills/platform-boundary/SKILL.md).

Procedures live in [Agent Skills](https://agentskills.io) under [`.agents/skills`](.agents/skills). Read the
matching skill before you start:

| Skill | Use it when |
| --- | --- |
| [fixing-a-bug](.agents/skills/fixing-a-bug/SKILL.md) | fixing any reported bug, review finding or regression (reproduce it in a failing test first) |
| [platform-boundary](.agents/skills/platform-boundary/SKILL.md) | adding or changing anything that talks to the OS, reviewing a diff for core/platform leaks, or auditing the crates |
| [rust-conventions](.agents/skills/rust-conventions/SKILL.md) | writing any Rust: naming, file size, errors, `unsafe`, docs, imports |
| [testing](.agents/skills/testing/SKILL.md) | designing a component so it can be tested in isolation, writing or running tests, which suites are safe to run locally |
| [manual-verification](.agents/skills/manual-verification/SKILL.md) | confirming a feature works on the live desktop: daemon, IPC, logs, screenshots |
| [commits-and-issues](.agents/skills/commits-and-issues/SKILL.md) | commits, plan checkboxes, filing or updating issues |

Ground rules:

- `mosaico-core` never depends on the `windows` crate, never contains `unsafe` or `#[cfg(target_os)]`, and never
  names OS concepts (`HWND`, `HMONITOR`, DWM, `ShowWindow`, `EVENT_*`, registry, `.exe`) in its types or docs.
- New OS functionality starts as a trait (or trait method) in core with a documented contract. The tiler, daemon and
  CLI call the trait, not concrete platform types or free functions.
- Logic that does not call the OS (parsing, formatting, comparing, deciding) belongs in core, even when only one
  platform uses it today.
- Every component must be testable in isolation: pass in the OS, clock, file system, network and IPC instead of
  reaching for them. See [testing](.agents/skills/testing/SKILL.md).
- Every bug fix starts with a test that reproduces the reported behaviour and fails on `main`. The fix then makes it
  pass without editing the test. See [fixing-a-bug](.agents/skills/fixing-a-bug/SKILL.md).
- Keep warnings at zero: `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings` must pass.
- Metrics must not regress: `cargo xtask metrics` must pass, and the release binary must stay under 2,000,000 bytes
  (`cargo xtask size`). See [testing](.agents/skills/testing/SKILL.md#metrics).
- Never run `cargo test` or `cargo test --workspace` unfiltered. Some suites start a real daemon that tiles the
  live desktop. See [testing](.agents/skills/testing/SKILL.md) for the safe set.
- Do not commit unless the user asks. Commits are GPG-signed conventional commits.
- The roadmap is in [`.plans/plan.md`](.plans/plan.md).
