# Phase 30 — Windows Installer & Winget Distribution

## Goal

Replace the current raw-binary distribution with a proper Windows
installer (MSI) and publish to winget as the single distribution
method. Users install, update, and uninstall mosaico exclusively
through `winget` and standard Windows mechanisms (Add/Remove Programs).

## Motivation

The current install.ps1 copies a bare .exe into `%LOCALAPPDATA%\mosaico`
and edits the user PATH. There is no uninstall path — the binary cannot
delete itself while running, the autostart Registry key is left behind,
and the config directory is orphaned. A proper installer solves all of
this by delegating lifecycle management to the OS.

## Requirements

1. **MSI installer** — produce an MSI (via WiX) in the release workflow.
2. **Install actions** — copy `mosaico.exe` to Program Files, add to
   PATH, optionally enable autostart.
3. **Uninstall actions** — stop the daemon, remove the binary, remove
   the PATH entry, remove the autostart Registry key. Leave the config
   directory (`~/.config/mosaico`) intact (standard convention).
4. **Upgrade support** — in-place upgrade without losing config.
5. **Winget manifest** — publish to the `microsoft/winget-pkgs`
   community repository so `winget install mosaico` works.
6. **Single distribution method** — winget is the only supported
   install/update/uninstall path going forward.

## Migration Strategy

Transition existing users from the raw-binary install to winget:

1. **`install.ps1` becomes a migration script** — detects the old
   `%LOCALAPPDATA%\mosaico` install, cleans it up (stops daemon,
   removes binary, PATH entry, autostart Registry key), then runs
   `winget install mosaico`.
2. **`mosaico update` triggers migration** — on first run after the
   MSI release, detects the old install layout, cleans it up, and
   invokes `winget upgrade mosaico`. After migration, `mosaico update`
   is a thin wrapper around `winget upgrade mosaico`.
3. **Drop raw .zip from releases** — once the MSI + winget pipeline is
   stable, stop attaching the bare .zip to GitHub releases. Only the
   MSI is published.

## Research

- **WiX v4** — Rust-friendly MSI authoring; `cargo-wix` crate can
  generate the WiX source from Cargo metadata.
- **Winget submission** — requires a manifest PR to
  `microsoft/winget-pkgs` with SHA256 hash of the installer URL.
  Can be automated with `wingetcreate` in the release workflow.

## Tasks

1. Add `cargo-wix` or hand-written WiX XML to produce an MSI.
2. Update the release workflow to build the MSI and attach it to the
   GitHub release.
3. Create a winget manifest and automate submission via `wingetcreate`
   in the release workflow.
4. Update `install.ps1` to detect old installs, clean up, and delegate
   to `winget install mosaico`.
5. Update `mosaico update` to detect old install layout, migrate, and
   delegate to `winget upgrade mosaico`.
6. After a few stable releases, remove the raw .zip from the release
   workflow.
7. Update docs / README with `winget install mosaico` as the canonical
   install method.
