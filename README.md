# Mosaico

A tiling window manager for Windows, written in Rust.

## Installation

### Windows

```powershell
irm https://mosaico.dev/install.ps1 | iex
```

### From source

```sh
cargo install --path .
```

## Usage

```sh
mosaico start    # Start the window manager
mosaico stop     # Stop the window manager
mosaico status   # Show current status
mosaico --help   # Show available commands
```

## Building from source

```sh
cargo build --release
```

## Running tests

```sh
cargo test
```

## License

[MIT](LICENSE)
