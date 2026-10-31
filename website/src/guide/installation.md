# Installation

## Windows

Run the following command in PowerShell to download and install the latest
release:

```powershell
irm https://raw.githubusercontent.com/jmelosegui/mosaico/main/docs/install.ps1 | iex
```

The installer downloads the latest release from GitHub and places `mosaico.exe`
and the settings app, `mosaico-settings.exe`, in `%LOCALAPPDATA%\mosaico`. It
also adds that directory to your user `PATH` so you can run `mosaico` from any
terminal.

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
