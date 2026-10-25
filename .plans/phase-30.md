# Phase 30: Winget Distribution

**Status:** Pending

**Goal:** Make `winget install jmelosegui.mosaico` the way to install mosaico,
in place of `docs/install.ps1`, without changing what gets installed: the same
release zip, the same `mosaico.exe`, the same config and data folders.

## Overview

Today a user runs `irm .../docs/install.ps1 | iex`. The script downloads
`mosaico-windows-amd64.zip` from the latest GitHub release, copies
`mosaico.exe` into `%LOCALAPPDATA%\mosaico`, unblocks it and adds that folder
to the user `PATH`. The same folder holds mosaico's data: the PID file,
`hidden-windows.json` and the `instances` folders. `mosaico update` replaces
the running executable in place with the one from the newest release (phase
22). There is no uninstall, and the script is a channel only this project
maintains.

winget installs a zip that holds a portable executable (`InstallerType: zip`,
`NestedInstallerType: portable`, since winget 1.4). It extracts the exe into a
folder of its own, puts a `mosaico` command on the `PATH`, records the version
in an Uninstall entry, and upgrades and uninstalls it. The release zip already
has that shape, so mosaico needs no installer of its own.

Three things stand between that and a working channel:

1. A portable package cannot be replaced while its executable runs. winget
   deletes the old file and copies the new one, so `winget upgrade` fails while
   the daemon runs, and so does an upgrade started by `mosaico update`, whose
   own process is that file. A failed upgrade also removes the `mosaico` link,
   or the package folder's `PATH` entry, before it gives up.
2. mosaico must know when winget installed it. `mosaico update` replaces the
   executable in place, and doing that to winget's file makes winget refuse
   to upgrade or uninstall the package without `--force`.
3. The first version on winget has to be one that already knows (2), so the
   code ships in a release before anything is submitted.

This phase makes mosaico aware of how it was installed, publishes each release
to winget, and turns the install script into a move to winget.

The zip winget downloads gets the Mark of the Web, and winget has it scanned,
but the executable it extracts from it carries none, so SmartScreen does not
prompt. Windows 11 Smart App Control is another matter: on devices that have it
on, it blocks unsigned executables wherever they come from. 30.5 signs the
executable for it.

The work splits into five steps, one PR each where there is code (see Tasks).
They go in order: 30.1 ships in a release, 30.2 submits that release by hand,
30.3 automates every release after it, and 30.4 sends users to winget once the
package is live. 30.5 is optional and can come at any time.

## Decisions for the Owner

1. **Run the daemon from a copy (recommended).** `mosaico start` launches the
   daemon from a copy of the executable in the data folder, one per version,
   instead of from the installed file. Nothing holds the installed file then,
   so `winget upgrade`, `winget upgrade --all` and an uninstall from Settings
   work while mosaico runs. The daemon keeps running the version it started
   with until it is restarted, and `mosaico doctor` and the bar say when a newer
   version is installed. This is the one place the phase goes beyond the
   channel. Without it, the docs have to tell users to run `mosaico stop`
   before every winget upgrade, and `winget upgrade --all` fails for mosaico
   while it runs.
2. **Keep today's install as the script's fallback.** On LTSC, Server, or
   machines where policy blocks the Store or winget, the script installs as it
   does today instead of failing.

The rest of the plan assumes both.

## Design Goals

- What is installed does not change: one `mosaico.exe`, config in
  `~/.config/mosaico`, data in `%LOCALAPPDATA%\mosaico`, autostart through the
  same `HKCU` Run entry. Only the channel that puts it there changes, and,
  with decision 1, where the daemon runs from.
- The release zip stays a release asset, unchanged. winget installs it, and
  `mosaico update` still uses it for installs that are not winget's.
- A winget install is only ever changed by winget. `mosaico update` never
  writes into winget's folder, and never runs winget itself, since its own
  process is the file winget would replace.
- Nothing stops the daemon for an upgrade that is not there: mosaico compares
  versions before it suggests anything.
- Every release reaches winget without a manual step after the first one, and
  a failure to submit never blocks the GitHub release.
- Core stays portable. Deciding how mosaico was installed, which version to
  offer and when to refresh the daemon's copy are pure functions in core over
  facts the platform reads. Core never names winget.
- Everything is testable: the decisions and the commands run against fakes,
  the script runs under Pester in CI, and the package is checked in a sandbox
  before it is submitted and from the community source after.

## The Package

| Field | Value |
| --- | --- |
| `PackageIdentifier` | `jmelosegui.mosaico` |
| `PackageVersion` | the release tag without its `v` |
| `Moniker` | `mosaico` |
| `InstallerType` / `NestedInstallerType` | `zip` / `portable` |
| `NestedInstallerFiles` | `mosaico.exe`, with `PortableCommandAlias: mosaico` |
| `Architecture` | `x64`, the only build today; ARM64 runs it under emulation |
| `InstallerUrl` | the release's `mosaico-windows-amd64.zip` |
| `License` | MIT |
| `Commands` | `mosaico` |

The manifests live in `microsoft/winget-pkgs`, not in this repository: that
repository is their source of truth, and each new version is made from the
last one there.

Where winget puts a portable package, to confirm in 30.2:

- the executable goes in
  `%LOCALAPPDATA%\Microsoft\WinGet\Packages\jmelosegui.mosaico_Microsoft.Winget.Source_8wekyb3d8bbwe\`,
  or under `%ProgramFiles%\WinGet\Packages\` with `--scope machine`, or where
  `--location` or the `portablePackageUserRoot` and `portablePackageMachineRoot`
  settings say. The folder name carries no version, so it stays the same
  across upgrades;
- with Developer Mode on or an elevated install, a `mosaico` link goes in
  `%LOCALAPPDATA%\Microsoft\WinGet\Links\`, which winget adds to the user
  `PATH`. Otherwise, the common case, the package folder itself goes on the
  user `PATH`;
- an Uninstall entry, `jmelosegui.mosaico_Microsoft.Winget.Source_8wekyb3d8bbwe`
  under `HKCU` (or `HKLM`), holds `InstallLocation` and `DisplayVersion`, and
  its `UninstallString` runs `winget uninstall --product-code` with that name.

## What Users Do

| Task | Command |
| --- | --- |
| Install | `winget install jmelosegui.mosaico`, then open a new terminal |
| Upgrade | `winget upgrade jmelosegui.mosaico` (or `--all`), then `mosaico stop` and `mosaico start` to run the new version |
| Uninstall | `mosaico autostart disable` and `mosaico stop`, then `winget uninstall jmelosegui.mosaico` or Settings > Apps |

Uninstalling leaves the config and data folders, as uninstallers usually do,
and the docs say where they are. winget cannot run a step of mosaico's own on
uninstall, so the docs list the two commands to run first. Without them, the
Run entry is left pointing at a file that is gone.

## Architecture

### Knowing how mosaico was installed

- The platform reads the facts: the Uninstall entry winget wrote for
  `jmelosegui.mosaico`, per user and per machine (`InstallLocation`,
  `DisplayVersion`), and where the running executable really is, with links
  and 8.3 names resolved. A new `FileProbe` method, next to `same_file`, says
  whether a file is inside a folder, by the platform's rules for paths.
- Core decides. `update::installed_by(facts)` is `Install::Package { location,
  version }` when the running executable is inside the package's
  `InstallLocation`, and `Install::Standalone` otherwise: the old script's
  folder, a `cargo` build, or a copy somewhere else.
- Tests on fakes cover a package install run through its link, through the
  folder on `PATH` and from the package folder; a standalone install while a
  package install also exists; a machine install; a `--location` install; and
  no package install at all.

### The daemon's copy (decision 1)

- `mosaico start` copies the running executable to
  `<data dir>\bin\<version>\mosaico.exe` when that file is missing, and starts
  the daemon from it. Each version has its own folder, so a copy is never
  written while a daemon runs from it. Copies of other versions that no process
  runs are removed at the next start, best effort.
- Core decides what to copy, where and what to remove, from the version and
  the folder listing; the platform copies, lists and launches. The launch path
  stops being `current_exe()` for the daemon.
- The Run entry and every command still name the installed executable, so
  autostart keeps working across upgrades.
- The daemon reports its version over IPC. `mosaico doctor` and the bar say
  when the installed executable is newer than the running daemon, and how to
  restart it.
- A named instance (`MOSAICO_INSTANCE`) keeps its copies in its own data folder.

### `mosaico update`

- `Install::Standalone`: as today. Once the package is live (30.4), it also
  says that mosaico is on winget and how to move to it.
- `Install::Package`: it never replaces the executable and never runs winget.
  It reads the newest version winget offers, from the package's folder in
  `microsoft/winget-pkgs` through the GitHub API (one request, with the HTTP
  client `update` already uses, parsed in core), and the latest GitHub tag. Then
  it prints one of:
  - mosaico is up to date;
  - winget offers vY: run `winget upgrade jmelosegui.mosaico`, then
    `mosaico stop` and `mosaico start`;
  - vX is released and on its way to winget, which still offers vY.
- `--force` prints the `winget install --force` command instead.
- The start banner and the bar announce, for a package install, only a version
  winget offers, with the winget command, so nobody is told to update before
  winget can.

### `mosaico doctor`

- How mosaico was installed, from where, and, with decision 1, the version
  the daemon runs against the one installed.
- A warning when another `mosaico.exe` is on the `PATH`, such as one left by
  the old script. The CLI passes `PATH` in, the check splits it with
  `std::env::split_paths`, and `FileProbe::same_file` keeps a link and its
  target from counting as two copies.
- A warning when a package install's `mosaico` command is missing from the
  `PATH`, which a failed winget upgrade leaves behind.

### The install script

`docs/install.ps1` keeps its URL, since the one-line install is published in
many places. Users also rerun it to update, so a rerun has to work. It runs in
Windows PowerShell 5.1, the default for `irm | iex`.

1. If winget is missing or blocked, it installs as it does today (decision 2),
   and says that winget is the way once it is available.
2. It runs `winget install --id jmelosegui.mosaico --exact --source winget
   --accept-source-agreements --accept-package-agreements
   --disable-interactivity`. It checks `$LASTEXITCODE`: 0, "no applicable
   upgrade" (`0x8A15002B`) and "already installed" (`0x8A150061`) are success,
   anything else stops the script with the old install untouched.
3. It finds the new executable from the Uninstall entry's `InstallLocation`
   and runs it by full path from then on, since the `PATH` of this session does
   not have it yet.
4. If the old script's install is there, it moves it over:
   - it notes whether a daemon runs and whether autostart is on and points at
     the old executable (`mosaico autostart status`);
   - it stops the daemon with the old executable's `mosaico stop`, so hidden
     windows are restored, and waits for the process to exit;
   - it removes `mosaico.exe` and `mosaico.exe.old` from
     `%LOCALAPPDATA%\mosaico` and keeps everything else there, which is
     mosaico's data;
   - it removes that folder from the user `PATH`, reading the raw value with
     `DoNotExpandEnvironmentNames`, keeping its `REG_EXPAND_SZ` type and
     matching whole entries only;
   - if autostart was on, it runs `mosaico autostart enable` with the new
     executable. One that was turned off in Task Manager stays as it is.
5. It starts the daemon again with the new executable if one was running, and
   prints `mosaico --version`.

A failure stops the script with a message and no `exit`, which under `iex`
would close the user's window. Every step before the old install is removed
leaves it working.

### Releases

- The `release` job sets `prerelease` from the tag (a tag with a `-` is a
  prerelease), never overwrites assets (`overwrite_files: false`), and fails
  when the tag is not `v` followed by the version in
  `crates/mosaico/Cargo.toml`.
- A `winget` job runs after it, for tags that are not prereleases, with
  `permissions: contents: read`. It runs `vedantmgoyal9/winget-releaser`,
  pinned to a commit SHA, with `identifier: jmelosegui.mosaico` and
  `installers-regex: '\.zip$'`. The action makes the new version's manifests
  from the last ones in `microsoft/winget-pkgs` and opens the pull request
  there.
- The pull request comes from a fork owned by a separate bot account
  (`fork-user`), with that account's classic token (`public_repo` and
  `workflow` scopes) in `WINGET_TOKEN`. So the token cannot push to this
  repository, whose `main` serves the install script.
- The job is not required for the release. A failed submission is retried with
  "Re-run failed jobs" only: rebuilding the zip would change the SHA-256 of a
  version winget may already have.
- The script's tests run in `ci.yml` on every pull request that touches the
  script, since users fetch it from `main` as soon as it merges. The asset name
  check moves there too.

## Edge Cases

1. **winget upgrades while mosaico runs.** With decision 1 nothing holds the
   installed file, so the upgrade works and the daemon keeps running the old
   version until it is restarted. Without it, the upgrade fails and the
   `mosaico` command drops off the `PATH` until one succeeds.
2. **A failed upgrade removes the command.** `mosaico doctor` warns, and the
   docs say a successful `winget upgrade` or `winget install --force` puts it
   back.
3. **Autostart.** The Run entry names the package file, not the link, since
   the link is what a failed upgrade removes. The package folder stays the
   same across upgrades, so the entry stays right.
4. **Two copies.** A user who installs with winget and keeps the old script's
   copy has two `mosaico.exe` on the `PATH`. `mosaico doctor` warns, and the
   script removes the old one.
5. **winget is behind GitHub.** Each winget-pkgs pull request waits for a
   moderator, often hours or days. `mosaico update`, the banner and the bar
   only offer what winget has.
6. **No winget.** The script falls back to today's install (decision 2).
7. **Machine scope.** Upgrading an install made with `--scope machine` needs
   an elevated prompt. The docs say to run winget elevated and then
   `mosaico start` from a normal prompt, since a daemon started elevated
   manages a different set of windows.
8. **Uninstall from Settings.** It runs `winget uninstall` without
   `mosaico stop` or `mosaico autostart disable`. With decision 1 the
   uninstall works while the daemon runs; the Run entry is left behind, and the
   docs say so.
9. **Smart App Control.** It blocks unsigned executables on the Windows 11
   devices that have it on. Until 30.5 the docs say so.
10. **Prereleases.** A tag with a `-` is published to GitHub as a prerelease
    and never submitted to winget.

## Later

- `mosaico update` running the winget upgrade for the user, from a copy of
  itself that waits for it to exit.
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
| `crates/mosaico-core/src/update/installed.rs` | `Install` and `installed_by()` |
| `crates/mosaico-core/src/update/offered.rs` | the newest version winget offers, parsed from the API |
| `crates/mosaico-core/src/update/daemon_copy.rs` | what to copy and remove for the daemon |
| `crates/mosaico-windows/src/package_entry.rs` | reading winget's Uninstall entry |
| `crates/mosaico-windows/src/` file probe and process launch | inside-folder check, copies, launching the daemon from its copy |
| `crates/mosaico/src/commands/update_package.rs` | `mosaico update` for a package install |
| `crates/mosaico/src/commands/start.rs`, `doctor_*.rs` | the daemon's copy, the install lines and warnings |
| `docs/install.ps1`, `tests/install/*.Tests.ps1` | the move to winget and its tests |
| `.github/workflows/release.yml`, `ci.yml` | the `winget` job, prereleases, the version check, the script tests |
| `README.md`, `website/src/guide/installation.md`, `website/src/README.md`, `docs/` | winget as the way to install, upgrade and uninstall |

## Tasks

Every step with code runs fmt, clippy, the safe test suites, `cargo xtask
metrics` and `cargo xtask size`, and records its metrics.

### 30.1 mosaico knows how it was installed

This ships in a release before anything is submitted to winget.

- [ ] Winget's Uninstall entry and the inside-folder check on Windows;
      `Install` and `installed_by()` in core, with tests for every case above
- [ ] The daemon's copy: copying, launching, cleaning up, the version over IPC,
      and tests on fakes, including a named instance (decision 1)
- [ ] The newest version winget offers, parsed in core, with tests for a
      package folder with several versions, none, and an API failure
- [ ] `mosaico update` for a package install, with tests for each message and
      `--force`; a test that it never replaces a package's executable
- [ ] The banner and the bar for a package install
- [ ] `mosaico doctor`: the install, the daemon's version, a second copy on the
      `PATH`, a missing command
- [ ] docs/cli.md, docs/daemon.md and the guide's CLI page

### 30.2 The first version on winget (by hand)

- [ ] Choose the package identifier and moniker (`jmelosegui.mosaico`,
      `mosaico`)
- [ ] Make the manifests for the release with 30.1, with `komac new` or
      `wingetcreate new`, from the release zip
- [ ] `winget validate` them and test them with winget-pkgs'
      `Tools/SandboxTest.ps1`: install, `mosaico --version`, `mosaico start`,
      uninstall
- [ ] Sign the contributor license agreement and submit them to
      `microsoft/winget-pkgs`
- [ ] Once merged, as a standard user and from the community source: record
      the install folder, the `PATH` entry or the link, and the Uninstall
      entry, with and without Developer Mode; upgrade from the previous
      version with the daemon running; uninstall from Settings > Apps
- [ ] Check that a winget install raises no SmartScreen prompt; on a device
      with Smart App Control on, if one is at hand, record what it does
- [ ] Check what a standard user's upgrade of a machine-scope install does:
      a UAC prompt or a failure

### 30.3 Every release on winget

- [ ] Prereleases, `overwrite_files: false` and the version check in the
      `release` job
- [ ] The `winget` job, pinned, read-only, not required for the release
- [ ] The bot account, its fork of `microsoft/winget-pkgs`, and `WINGET_TOKEN`
- [ ] Check it on the next release: the pull request is opened with the right
      URL and SHA-256

### 30.4 winget as the way to install

This merges only once a version with 30.1 is on winget.

- [ ] `docs/install.ps1` as the move to winget, with today's install as the
      fallback
- [ ] Pester tests in `ci.yml`, under Windows PowerShell 5.1, with `winget`
      mocked and a built `mosaico.exe` and Uninstall entry where the script
      looks: a fresh install, a move from the old install with the daemon
      running and autostart on, autostart turned off in Task Manager, a rerun,
      winget missing, and winget failing
- [ ] The asset name check moved to `ci.yml`
- [ ] `mosaico update` on a standalone install points to winget
- [ ] README, the guide's installation page and the docs: install, upgrade and
      uninstall with winget, a new terminal after installing, Settings > Apps,
      where the config and data folders are, and what to run before
      uninstalling
- [ ] Manual test: the one-line install on a machine with the old install and
      autostart on

### 30.5 Signing (optional)

- [ ] Apply to the SignPath Foundation, which signs open source projects for
      free from GitHub Actions builds, with the certificate in its name
- [ ] Its code signing policy page: team roles with MFA, and a privacy
      statement. mosaico checks GitHub for updates when it starts and downloads
      the community window rules, so say so, or let users turn both off
- [ ] Build, upload the unsigned `mosaico.exe`, have SignPath sign it, then zip,
      release and let winget hash the published zip. Each signing request waits
      for an approver, so a release now has one manual step; give it a timeout
- [ ] Check the signature with `Get-AuthenticodeSignature` in the workflow, and
      check `mosaico update` still installs the signed executable
- [ ] Update the docs' notes on SmartScreen and Smart App Control

- [ ] Update `.plans/plan.md`
