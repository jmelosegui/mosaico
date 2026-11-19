//! The `mosaico` table scripts call.

use mlua::serde::SerializeOptions;
use mlua::{Lua, LuaSerdeExt, MultiValue, Table, Value};

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{Command, IpcError, Response, ResponseStatus};

use crate::convert;
use crate::runtime::State;

#[path = "api_desktop.rs"]
mod desktop;
#[path = "api_events.rs"]
mod events;
#[path = "api_system.rs"]
mod system;

/// Puts the `mosaico` table in `lua`'s globals, its functions sharing
/// `state`, with `args` as `mosaico.args`.
pub fn install(lua: &Lua, state: &State, args: &[(String, String)]) -> mlua::Result<()> {
    let mosaico = lua.create_table()?;
    desktop::install(lua, &mosaico, state)?;
    events::install(lua, &mosaico, state)?;
    system::install(lua, &mosaico, state, args)?;
    lua.globals().set("mosaico", mosaico)?;
    Ok(())
}

/// How values cross into Lua: a missing value is `nil`, not a sentinel.
pub(crate) fn options() -> SerializeOptions {
    SerializeOptions::new()
        .serialize_none_to_null(false)
        .serialize_unit_to_null(false)
}

/// `value` as a Lua value.
pub(crate) fn to_lua(lua: &Lua, value: &serde_json::Value) -> mlua::Result<Value> {
    lua.to_value_with(value, options())
}

/// An event as a handler receives it.
pub(crate) fn event_value(lua: &Lua, event: &DesktopEvent) -> mlua::Result<Value> {
    to_lua(lua, &convert::event(event))
}

/// `nil, message`: how a call that could not do what it was asked says
/// why, without raising an error the script has to catch.
pub(crate) fn failure(lua: &Lua, message: impl Into<String>) -> mlua::Result<MultiValue> {
    let message = Value::String(lua.create_string(message.into())?);
    Ok(MultiValue::from_vec(vec![Value::Nil, message]))
}

/// `value` on its own.
pub(crate) fn success(value: Value) -> mlua::Result<MultiValue> {
    Ok(MultiValue::from_vec(vec![value]))
}

/// Sends `command`, and returns `true`, or `nil` and why not.
pub(crate) fn command(lua: &Lua, state: &State, command: &Command) -> mlua::Result<MultiValue> {
    let daemon = state.borrow().env.daemon.clone();
    match refused(daemon.send(command)) {
        Ok(_) => success(Value::Boolean(true)),
        Err(message) => failure(lua, message),
    }
}

/// A successful reply, or what went wrong as a script should read it.
pub(crate) fn refused(reply: Result<Response, IpcError>) -> Result<Response, String> {
    match reply {
        Ok(response) if response.status == ResponseStatus::Ok => Ok(response),
        Ok(response) => Err(response
            .message
            .unwrap_or_else(|| "the daemon refused it".into())),
        Err(IpcError::NotRunning) => Err("mosaico is not running".into()),
        Err(e) => Err(e.to_string()),
    }
}

/// Reads the field `name` of an optional table, if it is there.
pub(crate) fn field<T: mlua::FromLua>(
    table: Option<&Table>,
    name: &str,
) -> mlua::Result<Option<T>> {
    match table {
        Some(table) => table.get(name),
        None => Ok(None),
    }
}
