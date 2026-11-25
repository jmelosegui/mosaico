# Scripting

Mosaico runs Lua scripts that read the desktop, react to what happens on it
and act on it: float the meeting window, switch layouts when you dock, give a
lone window the whole screen, run a command every time you change workspace.

A script runs in its own process, as a client of the daemon, the same way
`mosaico action` does. Nothing runs inside the window manager, so a script
that fails, loops or crashes only stops itself.

## Your first script

```powershell
mosaico script new hello
```

This writes `hello.lua` in the scripts folder from a commented template and
opens it. Run it with:

```powershell
mosaico script run hello
```

A script that only calls functions runs to its end and exits. A script that
handles events with `mosaico.on` or sets timers keeps running until it calls
`mosaico.stop()` or you press `Ctrl+C`.

## Where scripts live

Everything is in the config folder (`%USERPROFILE%\.config\mosaico`, or the
folder `MOSAICO_CONFIG_DIR` names):

| Path | What it holds |
|------|---------------|
| `scripts\` | Your scripts. `mosaico script run <name>` runs `scripts\<name>.lua`. |
| `lib\` | Modules your scripts load with `require("name")`. `require` looks nowhere else. |
| `lib\mosaico.lua` | Type annotations of the `mosaico` API, for editors. It is never run. |
| `.luarc.json` | Points the Lua language server at `lib\`. |
| `logs\scripts\` | What scripts write when they run in the background. |

`mosaico script new` writes `lib\mosaico.lua` and `.luarc.json` when they are
missing. Open the config folder in an editor with a Lua language server, such
as VS Code with the Lua extension, and it completes and documents the API.

To reach the folder, run `mosaico script open`, or pick **Open scripts
folder** on the tray icon.

## Commands

| Command | What it does |
|---------|--------------|
| `mosaico script run <name or path> [key=value...]` | Runs a script. A name runs `scripts\<name>.lua`; a path with a folder in it, or ending in `.lua`, runs that file. |
| `mosaico script run <name> --background` | Runs it detached, without a console, writing to `logs\scripts\<name>.log`, and returns. |
| `mosaico script run <name> --fail-fast` | Stops at the first error in a handler or timer instead of reporting it and going on. |
| `mosaico script new <name>` | Creates a script from the template and opens it. |
| `mosaico script edit <name>` | Opens a script. |
| `mosaico script list` | Lists your scripts, marking those that start when you sign in. |
| `mosaico script add <path>` | Copies a script into the scripts folder. |
| `mosaico script open` | Opens the scripts folder. |
| `mosaico script autostart enable <name>` | Starts the script in the background whenever you sign in. |
| `mosaico script autostart disable <name>` | Stops doing that. |

Arguments after the name reach the script as `mosaico.args`: `mosaico script
run dock monitors=2 quiet=true` gives `mosaico.args.monitors == 2` and
`mosaico.args.quiet == true`.

A script that handles events and finds mosaico not running waits for it to
start. When mosaico stops or restarts, the script waits and goes on with the
same handlers, so a script started at sign-in keeps working.

To run a script from a key, bind it as an application in `keybindings.toml`:

```toml
[[launch]]
command = "mosaico.exe"
args = ["script", "run", "tidy"]
key = "T"
modifiers = ["alt", "ctrl"]
```

## The `mosaico` API

Workspaces are counted from 1, as the bar shows them. Windows and monitors are
named by the ids `mosaico.state()` and the events report.

Functions that ask mosaico for something return the answer, or `nil` and the
reason when they could not, for example because mosaico is not running or a
window has closed. They never raise an error your script has to catch:

```lua
local ok, why = mosaico.focus(window)
if not ok then mosaico.log("warn", "could not focus: " .. why) end
```

### Reading the desktop

| Function | Returns |
|----------|---------|
| `mosaico.state()` | `{ focused_window, focused_monitor, monitors, windows }` |
| `mosaico.windows(filter)` | The managed windows, or those whose `class`, `monitor`, `workspace` and `floating` equal the filter's and whose `title` contains its `title` |

Each window is `{ id, class, title, monitor, workspace, floating, visible,
minimized }`. Each monitor is `{ id, workspace, work_area, workspaces }`, where
`workspace` is the one on screen and each workspace is `{ index, layout,
monocle, windows, floating }`.

### Acting

| Function | What it does |
|----------|--------------|
| `mosaico.action(name)` | Runs an action by its name, such as `"focus-left"` or `"goto-workspace-3"`, on the focused window |
| `mosaico.focus(window)` | Focuses a window and brings it into view |
| `mosaico.send(window, workspace)` | Sends a tiled window to a workspace of its monitor and follows it |
| `mosaico.float(window [, on])` | Floats a window, or tiles it again with `false` |
| `mosaico.close(window)` | Asks a window to close |
| `mosaico.show(monitor, workspace)` | Shows a workspace on a monitor |
| `mosaico.layout(monitor, workspace, layout)` | Sets a workspace's layout: `"bsp"`, `"vertical-stack"` or `"three-column"` |
| `mosaico.monocle(monitor [, on])` | Turns monocle on or off on the workspace a monitor shows |

### Events and timers

| Function | What it does |
|----------|--------------|
| `mosaico.on(event, handler)` | Calls `handler(e)` with every event of that kind |
| `mosaico.wait(event [, filter] [, seconds])` | Waits for the next event of that kind whose fields equal the filter's, or returns `nil, "timed out"` |
| `mosaico.after(seconds, fn)` | Calls `fn` once, later. Returns a timer id |
| `mosaico.every(seconds, fn)` | Calls `fn` again and again. Returns a timer id |
| `mosaico.cancel(timer)` | Cancels a timer |
| `mosaico.sleep(seconds)` | Waits. Events that arrive meanwhile wait for the handlers |
| `mosaico.stop([code])` | Ends the script |

Handlers and timers run one at a time. While one runs, or while the script
waits with `wait` or `sleep`, events queue up and reach the handlers
afterwards. An error in a handler or timer is printed and the script goes on,
unless it runs with `--fail-fast`.

| Event | Fields |
|-------|--------|
| `window_managed` | `window`, `monitor`, `workspace`, `class`, `title` |
| `window_unmanaged` | `window` |
| `window_moved` | `window`, `monitor`, `workspace` |
| `window_title_changed` | `window`, `title` |
| `window_floating_changed` | `window`, `floating` |
| `focus_changed` | `window` (nil when nothing has focus), `monitor` |
| `workspace_changed` | `monitor`, `workspace` |
| `layout_changed` | `monitor`, `workspace`, `layout`, `monocle` |
| `monitor_added`, `monitor_removed` | `monitor` |
| `paused` | `paused` |
| `config_reloaded` | |

Every event also has `event`, its name. A misspelled event name is an error as
soon as the script runs, instead of a handler that never fires.

### Programs and data

| Function | What it does |
|----------|--------------|
| `mosaico.launch{ program, args... }` | Starts a program as the Run box would, and does not wait for it |
| `mosaico.exec{ program, args..., timeout = seconds }` | Runs a program and waits: `{ ok, code, stdout, stderr, timed_out }` |
| `mosaico.log([level,] text)` | Writes a line to the error stream, or to the log in the background |
| `mosaico.json.encode(value)` / `mosaico.json.decode(text)` | JSON |
| `mosaico.args` | The `key=value` arguments, as numbers, booleans or text |
| `print(...)` | Writes a line to the output, or to the log in the background |

Scripts run Lua 5.4 with its standard library, except what reaches a shell or
native code: `os.execute`, `io.popen`, `os.exit` and native modules are not
there. `mosaico.exec` runs a program from a list of arguments, with no shell
in between, and `mosaico.stop` ends the script.

## Examples

### Float meeting windows

```lua
mosaico.on("window_managed", function(e)
  if e.title:find("Meeting") then
    mosaico.float(e.window)
  end
end)
```

### Change layouts when you dock and undock

```lua
mosaico.on("monitor_added", function(e)
  mosaico.layout(e.monitor, 1, "three-column")
end)

mosaico.on("monitor_removed", function()
  local state = mosaico.state()
  if state then
    mosaico.layout(state.monitors[1].id, 1, "bsp")
  end
end)
```

### Monocle when a workspace holds one window

```lua
local function tidy(monitor)
  local state = mosaico.state()
  if not state then return end
  for _, m in ipairs(state.monitors) do
    if m.id == monitor then
      local ws = m.workspaces[m.workspace]
      mosaico.monocle(m.id, #ws.windows == 1)
    end
  end
end

mosaico.on("window_managed", function(e) tidy(e.monitor) end)
mosaico.on("window_moved", function(e) tidy(e.monitor) end)
mosaico.on("workspace_changed", function(e) tidy(e.monitor) end)
```

### Gather a program's windows on one workspace

```lua
for _, w in ipairs(mosaico.windows{ class = "Chrome_WidgetWin_1" } or {}) do
  if w.workspace ~= 3 and not w.floating then
    mosaico.send(w.id, 3)
  end
end
```

### Start your work session

```lua
mosaico.launch{ "wt.exe" }
local terminal = mosaico.wait("window_managed", { class = "CASCADIA_HOSTING_WINDOW_CLASS" }, 10)
if terminal then
  mosaico.send(terminal.window, 2)
end
```

### Run a command on every workspace switch

```lua
mosaico.on("workspace_changed", function(e)
  mosaico.exec{ "pwsh", "-NoProfile", "-File", "C:/scripts/on-workspace.ps1", tostring(e.workspace) }
end)
```

## Without Lua

`mosaico events` prints the same events as they happen, one per line, or as
JSON with `--json`, so any language can react to them:

```powershell
mosaico events --json focus_changed workspace_changed
```
