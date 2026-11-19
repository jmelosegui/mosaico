---@meta
-- Type annotations for the `mosaico` table, for editors that read Lua
-- language server annotations. mosaico writes this file when it is
-- missing; it is never run.

---@class mosaico.Window
---@field id integer
---@field class string
---@field title string
---@field monitor integer
---@field workspace integer Counted from 1.
---@field floating boolean
---@field visible boolean
---@field minimized boolean

---@class mosaico.Workspace
---@field index integer Counted from 1.
---@field layout "bsp"|"vertical-stack"|"three-column"
---@field monocle boolean
---@field windows integer[] Tiled windows, in layout order.
---@field floating integer[]

---@class mosaico.Monitor
---@field id integer
---@field workspace integer The workspace on screen, counted from 1.
---@field work_area { x: integer, y: integer, width: integer, height: integer }
---@field workspaces mosaico.Workspace[]

---@class mosaico.State
---@field focused_window integer?
---@field focused_monitor integer?
---@field monitors mosaico.Monitor[]
---@field windows mosaico.Window[]

---@class mosaico.Event
---@field event string The event's name, such as "focus_changed".
---@field window integer?
---@field monitor integer?
---@field workspace integer? Counted from 1.
---@field class string?
---@field title string?
---@field floating boolean?
---@field layout string?
---@field monocle boolean?
---@field paused boolean?

---@alias mosaico.EventName
---| "window_managed" | "window_unmanaged" | "window_moved"
---| "window_title_changed" | "window_floating_changed" | "focus_changed"
---| "workspace_changed" | "layout_changed" | "monitor_added"
---| "monitor_removed" | "paused" | "config_reloaded"

---@class mosaico.Ran
---@field ok boolean Whether it exited with code 0.
---@field code integer?
---@field stdout string
---@field stderr string
---@field timed_out boolean

---@class mosaico
mosaico = {}

---The desktop now.
---@return mosaico.State? state, string? why
function mosaico.state() end

---The managed windows, all or those that match.
---@param filter? { class?: string, title?: string, workspace?: integer, monitor?: integer, floating?: boolean }
---@return mosaico.Window[]? windows, string? why
function mosaico.windows(filter) end

---Runs an action by its name, as `mosaico action` and the keybindings name it.
---@param name string Such as "focus-left" or "goto-workspace-3".
---@return true? done, string? why
function mosaico.action(name) end

---@param window integer
---@return true? done, string? why
function mosaico.focus(window) end

---Sends a tiled window to a workspace of its monitor and follows it.
---@param window integer
---@param workspace integer Counted from 1.
---@return true? done, string? why
function mosaico.send(window, workspace) end

---@param window integer
---@param on? boolean Float it (default) or tile it again.
---@return true? done, string? why
function mosaico.float(window, on) end

---@param window integer
---@return true? done, string? why
function mosaico.close(window) end

---Shows a workspace on a monitor.
---@param monitor integer
---@param workspace integer Counted from 1.
---@return true? done, string? why
function mosaico.show(monitor, workspace) end

---@param monitor integer
---@param workspace integer Counted from 1.
---@param layout "bsp"|"vertical-stack"|"three-column"
---@return true? done, string? why
function mosaico.layout(monitor, workspace, layout) end

---@param monitor integer
---@param on? boolean
---@return true? done, string? why
function mosaico.monocle(monitor, on) end

---Calls `handler` with every event of this kind until the script stops.
---@param event mosaico.EventName
---@param handler fun(event: mosaico.Event)
function mosaico.on(event, handler) end

---Waits for the next event of this kind whose fields equal those of `filter`.
---@param event mosaico.EventName
---@param filter? table
---@param seconds? number
---@return mosaico.Event? event, string? why
function mosaico.wait(event, filter, seconds) end

---@param seconds number
function mosaico.sleep(seconds) end

---@param seconds number
---@param run fun()
---@return integer timer
function mosaico.after(seconds, run) end

---@param seconds number
---@param run fun()
---@return integer timer
function mosaico.every(seconds, run) end

---@param timer integer
---@return boolean cancelled
function mosaico.cancel(timer) end

---Ends the script.
---@param code? integer
function mosaico.stop(code) end

---Starts a program as the Run box would, and does not wait for it.
---@param argv string[] The program, then its arguments.
---@return true? done, string? why
function mosaico.launch(argv) end

---Runs a program without a shell and waits for it.
---@param argv string[]|{ timeout?: number } The program, then its arguments.
---@return mosaico.Ran? ran, string? why
function mosaico.exec(argv) end

---@param level "debug"|"info"|"warn"|"error"|string
---@param text? string
function mosaico.log(level, text) end

mosaico.json = {}

---@param value any
---@return string
function mosaico.json.encode(value) end

---@param text string
---@return any value, string? why
function mosaico.json.decode(text) end

---The `key=value` arguments the script was run with.
---@type table<string, string|number|boolean>
mosaico.args = {}
