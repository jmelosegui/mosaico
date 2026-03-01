# Phase 28: API Hygiene & Safety Audit

**Status:** Pending

**Goal:** Align public APIs and unsafe code with Rust API Guidelines and
industry best practices for documentation, error handling, and safety.

## Overview

Mosaico already follows many Rust conventions, but a focused audit will
improve long-term maintainability and safety. This phase applies the
Rust API Guidelines to the public surface area and to unsafe blocks in
`mosaico-windows`, and removes `unwrap`/`expect` from library code paths.

## Best Practice Sources

- Rust API Guidelines (naming, docs, errors, safety)
- Internal repo rules (no `unwrap`/`expect` in library code, document
  unsafe blocks with `// SAFETY:`)

## Scope

- Public types/functions in `mosaico-core`
- Unsafe blocks and FFI wrappers in `mosaico-windows`
- Error reporting for fallible operations

## Guidelines Applied

1. **Document public items** with clear `///` comments describing what
   and why, not how.
2. **Add `# Errors` sections** for any public function returning
   `Result`.
3. **Add `# Safety` sections** for `unsafe` functions and `// SAFETY:`
   justifications for unsafe blocks.
4. **Avoid `unwrap`/`expect` in library code**, use `Result` and `?`.
5. **Naming consistency**: follow Rust naming conventions for types,
   traits, functions, and modules.

## Candidate Areas

- `mosaico-windows/src/ctrl_c.rs`: replace `expect` with error returns.
- `mosaico-windows` unsafe blocks: ensure each has a `// SAFETY:` note.
- `mosaico-core` public APIs: fill missing docs and error sections.

## Modified Files

```
crates/
  mosaico-core/
    src/
      *.rs                  # Doc updates and error sections
  mosaico-windows/
    src/
      *.rs                  # SAFETY comments and error handling
```

## Tasks

- [ ] Audit public `mosaico-core` items for missing `///` docs
- [ ] Add `# Errors` sections where `Result` is returned
- [ ] Replace `unwrap`/`expect` in library code with proper errors
- [ ] Add `// SAFETY:` comments to every unsafe block in `mosaico-windows`
- [ ] Review naming consistency for public APIs
- [ ] Add/adjust tests if behavior changes
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Update `.plans/plan.md`
