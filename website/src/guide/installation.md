# Installation

## Windows

Run the following command in PowerShell to download and install the latest
release:

```powershell
irm https://raw.githubusercontent.com/jmelosegui/mosaico/main/docs/install.ps1 | iex
```

The installer downloads the latest release from GitHub and places the binary
in `%LOCALAPPDATA%\mosaico`. It also adds that directory to your user `PATH`
so you can run `mosaico` from any terminal.

## From Source

If you have the Rust toolchain installed, you can build and install directly:

```sh
cargo install --path .
```

Or build a release binary without installing:

```sh
cargo build --release
```

The binary will be at `target/release/mosaico.exe`.

## Verifying the Installation

After installing, verify that Mosaico is available:

```sh
mosaico --version
```

You should see the version number printed to the terminal.
