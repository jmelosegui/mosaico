# Installation

## Windows

### winget

```powershell
winget install jmelosegui.mosaico
```

winget installs the release's MSI for your user, without administrator rights.
Upgrade with `winget upgrade jmelosegui.mosaico` and remove it with
`winget uninstall jmelosegui.mosaico`.

### Install script

Or run the following command in PowerShell to download and install the latest
release:

```powershell
irm https://raw.githubusercontent.com/jmelosegui/mosaico/main/docs/install.ps1 | iex
```

The script installs the same MSI. Both place `mosaico.exe` and the settings
app, `mosaico-settings.exe`, in `%LOCALAPPDATA%\Programs\mosaico`, add that
folder to your user `PATH` so you can run `mosaico` from any terminal, and add
**Mosaico Settings** to the Start menu. Mosaico appears in **Installed apps**,
where you can uninstall it; uninstalling stops the daemon and removes its
autostart entry.

Earlier versions of the script copied the binaries into `%LOCALAPPDATA%\mosaico`.
Running the new script moves such a copy to the installer: it removes the old
binaries and their `PATH` entry and keeps your autostart setting and a running
daemon. Your configuration is not touched.

### Portable zip

Each release also has `mosaico-windows-amd64.zip` with both executables, for
installing without the MSI. Keep them in the same folder, and add it to your
`PATH` yourself.

## From Source

If you have the Rust toolchain installed, you can build and install directly:

```sh
cargo install --path crates/mosaico
cargo install --path crates/mosaico-settings
```

Or build the release binaries without installing:

```sh
cargo build --release -p mosaico -p mosaico-settings
```

The binaries will be at `target/release/mosaico.exe` and
`target/release/mosaico-settings.exe`. Keep them in the same folder:
`mosaico settings` looks for the settings app beside `mosaico.exe`.

## Verifying the Installation

After installing, verify that Mosaico is available:

```sh
mosaico --version
```

You should see the version number printed to the terminal.
