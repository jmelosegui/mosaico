# Phase 30: Winget Distribution

**Status:** Pending

**Goal:** Make `winget install jmelosegui.mosaico` the way to install mosaico,
in place of `docs/install.ps1`, without changing what gets installed: the same
release zip, the same `mosaico.exe`, the same config and data folders.

## Overview

Today a user runs `irm .../docs/install.ps1 | iex`. The script downloads
`mosaico-windows-amd64.zip` from the latest GitHub release, copies
`mosaico.exe` into `%LOCALAPPDATA%\mosaico`, unblocks it and adds that folder
to the user `PATH`. `mosaico update` replaces the running executable in place
with the one from the newest release (phase 22). There is no uninstall, and
the script is a channel only this project maintains.

winget installs a zip that holds a portable executable (`InstallerType: zip`,
`NestedInstallerType: portable`, since winget 1.4). It extracts the exe into a
folder of its own, puts a `mosaico` command on the `PATH`, records the version,
and upgrades and uninstalls it. The release zip already has that shape, so
mosaico needs no installer of its own. This phase publishes each release to
winget, makes `mosaico update` hand a winget install over to winget, and turns
the install script into a move to winget.

Like the script, winget downloads the zip itself, not through a browser, so
the executable carries no Mark of the Web and SmartScreen does not prompt. The
one check it does not avoid is Windows 11 Smart App Control, which blocks
unsigned executables wherever they come from; 30.5 signs the executable for
it.

The work splits into five steps, one PR each where there is code (see Tasks).
30.1 is done by hand once; 30.2 to 30.4 can merge in any order after it; 30.5
is optional and can come at any time.

## Design Goals

- What is installed does not change: one `mosaico.exe`, config in
  `~/.config/mosaico`, data in `%LOCALAPPDATA%\mosaico`, autostart through the
  same `HKCU` Run entry. Only the channel that puts it there changes.
- The release zip stays a release asset, unchanged. winget installs it, and
  `mosaico update` still uses it for installs made by the old script.
- A winget install is only ever changed by winget, so winget's record of the
  installed version is always right: `mosaico update` never writes into
  winget's folder.
- The daemon is never left stopped by an update: it is stopped before winget
  replaces the executable, which winget cannot do while it runs, and started
  again afterwards whatever happened.
- Every release reaches winget without a manual step after the first one, and
  a failure to submit never blocks the GitHub release.
- Core stays portable: deciding how mosaico was installed is a pure function
  in core over paths the platform gives it, and running the package manager is
  a contract in core with a Windows implementation. Core never names winget.
- Everything is testable: the decision and the update flow run against fakes,
  the script's move runs in CI against a fake old install, and the package is
  checked by installing it in a clean sandbox before it is submitted.

## The Package

| Field | Value |
| --- | --- |
| `PackageIdentifier` | `jmelosegui.mosaico` |
| `Moniker` | `mosaico`, so `winget install mosaico` finds it |
| `InstallerType` / `NestedInstallerType` | `zip` / `portable` |
| `NestedInstallerFiles` | `mosaico.exe`, with `PortableCommandAlias: mosaico` |
| `Architecture` | `x64` (the only build today) |
| `InstallerUrl` | the release's `mosaico-windows-amd64.zip` |
| `MinimumOSVersion` | `10.0.17763.0`, the oldest Windows winget runs on |
| `License` | MIT |
| `Commands` | `mosaico` |

The manifests live in `microsoft/winget-pkgs`, not in this repository: that
repository is their source of truth, and each new version is made from the
last one there.

Where winget puts a portable package, to confirm in 30.1:

- per user (the default): the executable in
  `%LOCALAPPDATA%\Microsoft\WinGet\Packages\jmelosegui.mosaico_<source>\` and a
  link in `%LOCALAPPDATA%\Microsoft\WinGet\Links\`, a folder winget adds to the
  user `PATH`;
- per machine (`--scope machine`): the same layout under
  `%ProgramFiles%\WinGet\`.

The package folder carries no version, so its path stays the same across
upgrades, and an autostart entry made from it keeps working.

## What Users Do

| Task | Command |
| --- | --- |
| Install | `winget install jmelosegui.mosaico` |
| Upgrade | `mosaico update`, or `winget upgrade jmelosegui.mosaico` with the daemon stopped |
| Uninstall | `mosaico autostart disable`, `mosaico stop`, then `winget uninstall jmelosegui.mosaico` |

Uninstalling leaves the config and data folders, as uninstallers usually do,
and the docs say where they are. winget cannot run a step of mosaico's own on
uninstall, so the docs list the two commands to run first. Without them, the
uninstall fails on the running executable, or a Run entry is left pointing at
a file that is gone.

## Architecture

### Knowing how mosaico was installed (core)

- `update::Channel`: `SelfReplace`, as today, or `PackageManager`: the
  executable belongs to the system's package manager.
- `update::channel(exe, roots)`: a pure function. `exe` is the running
  executable's path, and `roots` are the folders the platform says its package
  manager installs into. An executable under one of them, compared without
  regard to case and after resolving `.`, `..` and links, is the package
  manager's. Anything else, the old script's folder, a `cargo` build or a copy
  somewhere else, replaces itself.
- Tests cover the old script's folder, each package root, a link in the
  package manager's link folder, a build in `target`, a path that only starts
  with a root's name, and different case.

### Running the package manager (contract in core)

```rust
/// The system's package manager, for an install it made.
///
/// # Contract for platform implementations
/// - `roots` lists the folders packages are installed into for this user
///   and for the machine; empty when there is no package manager.
/// - `upgrade` installs the newest version the package manager knows of,
///   without asking the user anything, and returns once it is done. It
///   reports whether a newer version was installed, whether none was
///   available yet, and any failure with the package manager's own output.
pub trait PackageManager {
    fn roots(&self) -> Vec<PathBuf>;
    fn upgrade(&self) -> Result<Upgraded>;
}
```

On Windows, `roots` are the two `WinGet\Packages` folders above, and
`upgrade` runs `winget upgrade --id jmelosegui.mosaico --exact --silent
--disable-interactivity --accept-source-agreements --accept-package-agreements`
and reads its exit code. The CLI gets it through its `Context`, like the
`Installer` it already has, and tests give it a fake.

### `mosaico update`

1. Find the newest release tag, as today.
2. Decide the channel. For `SelfReplace`, everything goes on as today.
3. For `PackageManager`:
   - stop the daemon if it runs;
   - run `upgrade`;
   - start the daemon again if it ran before, whatever the upgrade did;
   - report what happened.

   A release reaches winget only once its manifest is merged, often hours
   after the GitHub release. So when GitHub has a newer tag and winget has no
   upgrade yet, `mosaico update` says that the release is on its way to winget
   and to try again later. It does not fall back to replacing the executable.
4. `--force` reinstalls through winget too.

`mosaico doctor` gains one line: how mosaico was installed and from where.
If another `mosaico.exe` is on the `PATH`, such as one left by the old script,
it warns that the copy on the `PATH` may not be the one that runs.

### The install script

`docs/install.ps1` keeps its URL, since the one-line install is published in
many places, but it no longer installs anything itself:

1. Without winget, it says how to get it (App Installer from the Microsoft
   Store) and where the release zip is, and stops.
2. If the old script's install is there, it moves it over:
   - it stops the daemon;
   - it removes `mosaico.exe` and `mosaico.exe.old` from
     `%LOCALAPPDATA%\mosaico` and keeps everything else there, which is
     mosaico's data;
   - it removes that folder from the user `PATH`;
   - it notes whether autostart pointed at the old executable.
3. It runs `winget install --id jmelosegui.mosaico --exact`.
4. If autostart pointed at the old executable, it runs `mosaico autostart
   enable` with the new one, so the Run entry names the winget path.
5. It prints `mosaico --version` from the new install.

Each step says what it does, and a failure stops the script before the next
step, with the old install either still there or already gone, never half
removed. After a few releases the docs stop mentioning the script, and the
script stays for whoever still has the old line.

### Releases

A `winget` job runs after the `release` job of `.github/workflows/release.yml`,
for `v*` tags that are not prereleases. It runs
`vedantmgoyal2009/winget-releaser`, which makes the new version's manifests
from the last ones in `microsoft/winget-pkgs`, with the new URL and SHA-256,
and opens the pull request there. It needs a `WINGET_TOKEN` secret: a classic
personal access token with the `public_repo` scope, for an account with a fork
of `microsoft/winget-pkgs`. The job is not required for the release to succeed,
so a failed submission shows up as a failed job and can be retried.

The `publish-install-script` job checks the new script instead of the old one:
that it names the package identifier and the old install folder.

## Edge Cases

1. **The daemon runs during `winget upgrade` or `winget uninstall`.** winget
   cannot replace or remove a running executable. `mosaico update` stops the
   daemon first; the docs say to run `mosaico stop` before running winget
   yourself.
2. **Autostart.** The Run entry holds the path `mosaico autostart enable` was
   run from. The package folder does not change across upgrades, so the entry
   stays right. The script moves an old entry over (step 4 above).
3. **Two copies.** A user who installs with winget and keeps the old script's
   copy has two `mosaico.exe` on the `PATH`. `mosaico doctor` warns about it,
   and the script removes the old one.
4. **winget is behind GitHub.** See `mosaico update` step 3.
5. **No winget.** Windows 10 before 1809, LTSC and Server images may lack it.
   The script says how to get it, and the release zip still works for a
   manual install.
6. **A custom portable root.** winget can be set to install portable packages
   somewhere else (`portablePackageUserRoot`). Such an install is taken for a
   copy of its own and replaces itself, which works but leaves winget's
   record behind. The docs say so; reading winget's settings is left for
   later.
7. **Machine scope.** An install made with `--scope machine` lives under
   `%ProgramFiles%`. Upgrading it needs an elevated prompt; `mosaico update`
   reports winget's failure and says to run it elevated.
8. **Smart App Control.** It blocks unsigned executables on the Windows 11
   devices that have it on, wherever they came from. Until 30.5 the docs say
   so.

## Later

- An MSI, if mosaico ever needs install or uninstall steps of its own: stopping
  the daemon and removing the Run entry on uninstall.
- An ARM64 build, as a second installer in the same manifest.
- A Scoop bucket.
- The Microsoft Store, whose MSIX packages Microsoft signs. That would change
  how mosaico is installed: autostart and its data folder work differently for
  a packaged app.

## New and Modified Files

| File | Change |
| --- | --- |
| `crates/mosaico-core/src/update/channel.rs` | `Channel` and `channel()` |
| `crates/mosaico-core/src/update/package_manager.rs` | `PackageManager` contract and `Upgraded` |
| `crates/mosaico-windows/src/package_manager.rs` | winget roots and `winget upgrade` |
| `crates/mosaico/src/commands/update.rs` | the package manager path |
| `crates/mosaico/src/commands/doctor_*.rs` | the install line and the second-copy warning |
| `docs/install.ps1` | the move to winget |
| `.github/workflows/release.yml` | the `winget` job, the script check |
| `README.md`, `website/src/guide/installation.md`, `website/src/README.md`, `docs/` | winget as the way to install, upgrade and uninstall |

## Tasks

Every step with code runs fmt, clippy, the safe test suites and
`cargo xtask metrics`, and records its metrics.

### 30.1 The first version on winget (by hand)

- [ ] Choose the package identifier and moniker (`jmelosegui.mosaico`,
      `mosaico`)
- [ ] Make the manifests for the latest release with `komac new` or
      `wingetcreate new`, from the release zip
- [ ] `winget validate` them, then in Windows Sandbox install with
      `winget install --manifest`, run `mosaico --version` and `mosaico start`,
      upgrade from the previous release's manifest, and uninstall; record where
      winget put the executable and the link
- [ ] Check that a winget install raises no SmartScreen prompt, and what Smart
      App Control does with it where it is on
- [ ] Submit them to `microsoft/winget-pkgs` and see the pull request merged

### 30.2 Every release on winget

- [ ] The `winget` job in the release workflow, for `v*` tags that are not
      prereleases, not required for the release
- [ ] The `WINGET_TOKEN` secret and a fork of `microsoft/winget-pkgs` for it
- [ ] Check it on the next release: the pull request is opened with the right
      URL and SHA-256

### 30.3 `mosaico update` for winget installs

- [ ] `update::Channel` and `channel()` in core, with tests for every path in
      the list above
- [ ] The `PackageManager` contract in core, its Windows implementation, and a
      fake for the CLI tests
- [ ] The package manager path in `mosaico update`, with tests: daemon running
      or not, upgraded, nothing new yet, a failure, `--force`; the daemon is
      always started again when it ran before
- [ ] The install line and the second-copy warning in `mosaico doctor`
- [ ] docs/cli.md and the guide's CLI page
- [ ] Manual test: `mosaico update` on a winget install, with the daemon running

### 30.4 winget as the way to install

- [ ] `docs/install.ps1` as the move to winget, with each step above
- [ ] A CI job that runs the script against a fake old install in a temporary
      `LOCALAPPDATA`, with winget stubbed: the old executable is gone, the data
      files stay, the `PATH` entry is gone, winget was called with the package
      identifier, and autostart is moved over
- [ ] The release workflow's script check updated
- [ ] README, the guide's installation page and the docs: install, upgrade and
      uninstall with winget, where the config and data folders are, and what to
      run before uninstalling
- [ ] Manual test: the one-line install on a machine with the old install and
      autostart on

### 30.5 Signing (optional)

- [ ] Apply to the SignPath Foundation, which signs open source projects for
      free from GitHub Actions builds, with the certificate in its name
- [ ] The code signing policy page it asks for
- [ ] Sign `mosaico.exe` in the release workflow before it is zipped, so the
      zip, its SHA-256 and the winget manifest all carry the signed executable
- [ ] Check the signature with `Get-AuthenticodeSignature` in the workflow, and
      check `mosaico update` still installs the signed executable
- [ ] Update the docs' notes on SmartScreen and Smart App Control

- [ ] Update `.plans/plan.md`
