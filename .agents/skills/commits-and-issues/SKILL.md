---
name: commits-and-issues
description: Commit, plan and issue conventions for mosaico. Use when committing, updating the roadmap, or filing and updating GitHub issues.
---

# Commits and issues

## Commits

- Do not commit unless the user explicitly asks.
- Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, with a scope when useful
  (`fix(tiling): ...`).
- Explain why in the message, not just what.
- All commits are GPG-signed. Never use `--no-gpg-sign`.
- Before committing, run the safe suite from the [testing](../testing/SKILL.md) skill.
- Update plan checkboxes in `.plans/` in the same commit that completes the work.
- A bug fix is two commits: the reproducing test first, then the fix. See
  [fixing-a-bug](../fixing-a-bug/SKILL.md).

## Issues

- Search existing issues first (`gh issue list --state all`).
- State the problem with concrete file and symbol references, then a **Proposal** section.
- Issues about missing core contracts link to the umbrella issue #41 with `Part of #41.` and are added to its
  checklist. See [platform-boundary](../platform-boundary/SKILL.md).
