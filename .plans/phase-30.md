# Phase 30: Windows Installer & Winget Distribution

## Overview

Releases ship two executables, `mosaico.exe` and `mosaico-settings.exe`.
Copying them out of a zip leaves no uninstall path, no Start menu entry for
the settings app, and an autostart entry and PATH change that outlive the
binaries. Each release now also publishes a per-user MSI, which winget
installs, upgrades and removes, and which the install script uses too. The
zip stays as a portable download.

## Decisions

- **MSI, not winget's zip/portable type.** A portable install cannot run
  code around its file changes, so it cannot stop the daemon gracefully
  (which puts hidden windows back), cannot move an open settings app
  aside, and leaves the autostart entry behind on uninstall. The MSI runs
  mosaico itself at each step, through hidden `installer-hook` commands.
- **Per user, no elevation.** Installs into
  `%LOCALAPPDATA%\Programs\mosaico`, records the folder as `InstallFolder`
  under `HKCU\Software\mosaico`, adds the folder to the user PATH and adds
  a **Mosaico Settings** Start menu shortcut.
- **WiX 5** (dotnet tool), not 6, which needs its maintenance fee EULA
  accepted. The package is `packaging/windows/mosaico.wxs`, built with
  `cargo xtask msi`.
- **UpgradeCode `BCE5918A-87F4-4D50-B087-CD5305C8ABE6` never changes.** It
  ties versions together for major upgrades, and winget matches an
  installed mosaico on it.
- **Mosaico runs the hooks, not the MSI.** `MajorUpgrade` is scheduled
  after `InstallInitialize`, and Restart Manager is disabled
  (`MSIRESTARTMANAGERCONTROL=Disable`): WiX's CloseApplications runs too
  late to close the settings app, and Restart Manager would end the
  daemon without restoring windows.
  - Before an upgrade or reinstall, from the old executable: stop the
    daemon, leaving a restart note if it ran, and rename a running
    settings app to `.exe.old`.
  - After an install: start the daemon if the note is there.
  - Before an uninstall: stop the daemon, delete `.exe.old` leftovers,
    rename a running settings app aside, remove the autostart entry when
    it runs this executable, and clear the note.
- **`mosaico update` keeps working.** A copy in the MSI's folder downloads
  the release's MSI and runs it (`REINSTALL=ALL` for `--force` on the same
  version), so Installed apps shows the right version. Other copies
  replace their own files as before.
- **The zip stays.** Portable users and older `mosaico update` binaries
  still use it; dropping it is a later decision.

## Platform boundary

- `Installer` (core) gained `package_name`, `package_folder`,
  `run_package` and `set_aside_if_running`, with defaults for platforms
  without an installer package. `update::is_packaged` and
  `update::download_package` are core logic; `RestartMarker` is the
  restart note, kept in the instance's data directory.
- `mosaico-windows` implements them with the MSI names, the
  `InstallFolder` registry value, `msiexec` and a sharing-violation check.

## Found along the way

Detached children (the daemon started by `mosaico start`, the settings
app) inherited the launcher's standard output handles, so anything reading
the launcher's output (`$x = mosaico start`, or the MSI's quiet exec)
waited until the daemon exited. `process::spawn_detached` now clears the
inherit flag on the launcher's standard handles first.

## Tasks

- [x] `packaging/windows/mosaico.wxs` and `cargo xtask msi`
- [x] Core contracts for installer packages, `is_packaged`,
      `download_package` and `RestartMarker`
- [x] Windows implementation of the package methods
- [x] `mosaico installer-hook before-upgrade|after-install|before-uninstall`
- [x] `mosaico update` runs the MSI for an installed copy
- [x] Fix detached children holding the launcher's output open
- [x] Release workflow builds the MSI and attaches it with the zip
- [x] CI builds the MSI and installs and uninstalls it
- [x] Release workflow job that opens the winget-pkgs pull request
      (`vedantmgoyal9/winget-releaser`, needs the `WINGET_TOKEN` secret)
- [x] `packaging/winget/new-manifests.ps1` for the first submission
- [x] `install.ps1` installs the MSI and moves a zip install over to it
- [x] Docs: README, installation guide, `mosaico update`
- [ ] Tag the first release with an MSI
- [ ] Submit the first version of `jmelosegui.mosaico` to winget-pkgs by
      hand; later versions go through the release workflow
- [ ] Sign the MSI and executables (SmartScreen warns about unsigned
      downloads)
- [ ] Decide when to stop publishing the zip
