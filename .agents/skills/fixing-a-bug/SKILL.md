---
name: fixing-a-bug
description: How to fix a bug in mosaico. A test that reproduces the reported behaviour must fail on main before any product code changes, and the fix must make it pass without editing the test. Use for every bug report, review finding or regression.
---

# Fixing a bug

The rule: **no product change until a test reproduces exactly what was reported and fails on `main` in that
way.** Then change the product code, without touching the test, until it passes.

A test that fails for some other reason (it does not compile, it asserts the wrong thing, it fails on a
setup detail) proves nothing about the bug. A test written after the fix, or edited until it passes, proves
nothing either.

Related skills: [testing](../testing/SKILL.md) (where tests live, the safe suite, designing for isolation),
[platform-boundary](../platform-boundary/SKILL.md), [commits-and-issues](../commits-and-issues/SKILL.md).

## 1. Understand the report

- Read the whole issue and every comment.
- Write down the **observed** behaviour and the **expected** behaviour, as concretely as the report allows:
  which effect, reply, message, window position or log line, after which sequence of inputs.
- Check whether the behaviour is intended (a comment in the code, a test that asserts it, a decision in an
  issue). An intended behaviour needs a decision from the user, not a fix.

## 2. Write the failing test

Put the test at the **lowest layer that can show the bug**, and drive it through that layer's public entry
point with fakes for everything else:

| Where the bug is | Test it with |
| --- | --- |
| Tiler decisions | the tiler `Harness` in `crates/mosaico-core/src/tiler/tests.rs` (fake desktop, controlled clock), asserting on the effects |
| Coalescing, echo filtering | `Coalescer` / `Expectations` directly, or the harness with them in the loop |
| IPC framing, requests, `serve` | the fake transports in `crates/mosaico-core/src/ipc/tests/` |
| CLI commands | `World` and `Harness` in `crates/mosaico/src/commands/test_support.rs` |
| Config loading and migration | a temp directory passed to the loader |
| Platform code in `mosaico-windows` | a unit test of the extracted pure function; see below |

How it is written:

- Reproduce the reported sequence as closely as possible, trimmed to the minimum. Keep the details that
  matter (ordering, timing, which monitor, which hiding mode).
- Assert the **expected** behaviour, not the absence of one symptom. For a ghost effect, assert the full
  list of effects; for a wrong reply, assert the status and code.
- Name the test after the behaviour, and reference the issue in a comment: `// #71`.
- Add the near miss that must keep working, when there is one.

Run it on `main` and read the failure (`cargo test --lib <name>`, or `cargo test -p mosaico --bins <name>` for a
CLI command test).
The failure message must show the reported behaviour. If it fails differently, fix the test first.

### When the bug cannot be reproduced without a desktop

That means a seam is missing (see "Design for isolation" in the [testing](../testing/SKILL.md) skill). Add the
seam first, as a refactor with no behaviour change and its own commit, then write the failing test against
it. A desktop integration test (`crates/mosaico/tests/`) is the last resort, and only with the user's go-ahead
because it tiles the live desktop.

## 3. Fix

- Make the smallest change in the product code that makes the test pass.
- **Do not edit the reproducing test.** If it turns out to be wrong, stop, fix the test, confirm it still
  fails on `main` for the reported reason, and start the fix again.
- Fix the root cause. If several call sites share it, cover each with a test.
- Keep the change in the right crate (see [platform-boundary](../platform-boundary/SKILL.md)).

## 4. Prove the fix

- Commit the test on its own first (`test: reproduce #71 ...`), then the fix (`fix(tiler): ...`). This lets a
  reviewer check out the first commit and see the test fail.
- Confirm the fix did not touch the test:

  ```sh
  git diff <test-commit> HEAD -- <test file>
  ```

  The output must be empty.
- The new test passes, and so does the rest of the safe suite from the [testing](../testing/SKILL.md) skill.
- Every new branch in the fix is covered by a test, including the path that must keep working.

## 5. Pull request

- Title: a conventional commit, e.g. `fix(tiler): drop held effects when a window leaves the screen`.
- Body: `Fixes #<issue>` on its own line, what was wrong, what changed, the name of the reproducing test,
  and that it **fails on main** with the failure message it gave.
- Flag any behaviour change beyond the bug clearly.
