# Other Bars

mosaico's own bar is the default, but another bar can show your workspaces
instead. Pick it with `[bar] provider` in `config.toml`, or on the Bar page of
the [Settings Window](settings.md#bar), and mosaico's bar steps aside.

## YASB

[YASB](https://github.com/amnweb/yasb) draws its bar with widgets that show
mosaico's workspace and focused window, and run mosaico's actions on a click.

1. Start YASB, then run:

   ```sh
   mosaico bar setup yasb
   ```

2. Copy the widgets it prints under `widgets:` in YASB's `config.yaml`
   (`~/.config/yasb/config.yaml`), and name them in one of your bars:

   ```yaml
   bars:
     primary-bar:
       window_flags:
         windows_app_bar: true   # Windows tile below the bar
       widgets:
         left: ["mosaico_workspace", "mosaico_layout"]
         center: ["mosaico_window"]
         right: ["clock"]
   ```

YASB reloads its config when you save it. Once YASB is running,
`mosaico bar setup yasb` also sets `provider = "yasb"` in `config.toml`. While
it is not running, mosaico keeps its own bar, so you are never left without
one.

| Widget | Shows | Left click | Right click |
|--------|-------|------------|-------------|
| `mosaico_workspace` | The focused workspace | Next workspace | Previous workspace |
| `mosaico_layout` | `layout` | Next layout | Monocle on or off |
| `mosaico_window` | The focused window's title | | |

The widgets read mosaico once a second, and show the monitor that has the
focus. `mosaico doctor` says whether YASB is running.

To go back to mosaico's bar, set `provider = "mosaico"`.

## Any Bar

A bar that can run a command can show mosaico too. Set
`provider = "external"`, then use these commands from the bar:

```sh
mosaico subscribe --field focus.workspace --wait   # A line each time it changes
mosaico subscribe --full --wait                    # The whole state as JSON, each time it changes
mosaico query --field focus.window.title           # One value, for a bar that runs a command on a timer
mosaico run goto-workspace-3 --monitor 1           # A click: an action on a monitor
```

`--wait` keeps the command running while mosaico restarts. See
[`mosaico query`](cli.md#mosaico-query), [`mosaico subscribe`](cli.md#mosaico-subscribe)
and [`mosaico run`](cli.md#mosaico-run-action) for the topics, the fields and
the actions.

Windows tile into the space Windows leaves free. For a bar that does not keep
its own space, keep it with `[bar.reserve]` (see
[Status Bar](status-bar.md#space-for-another-bar)), or let the bar ask while it
runs:

```sh
mosaico subscribe --field focus.workspace --wait --reserve top=32
```
