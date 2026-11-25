-- A mosaico script. Run it with:
--
--   mosaico script run NAME
--
-- It runs in its own process and talks to mosaico like the CLI does, so
-- an error here never stops the window manager. Workspaces are counted
-- from 1, as the bar shows them. Every function is described at
-- https://jmelosegui.com/mosaico/guide/scripting.html

-- What is on the desktop now.
local state, why = mosaico.state()
if state then
  print("focused window: " .. tostring(state.focused_window))
else
  print("could not read the desktop: " .. why)
end

-- React to what happens. A script with handlers keeps running until it
-- calls mosaico.stop() or you press Ctrl+C.
mosaico.on("workspace_changed", function(event)
  print("monitor " .. event.monitor .. " shows workspace " .. event.workspace)
end)
