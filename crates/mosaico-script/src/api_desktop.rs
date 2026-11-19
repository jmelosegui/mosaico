//! `mosaico.state`, `mosaico.windows`, `mosaico.action` and the functions
//! that act on a window, monitor or workspace by its id.

use std::str::FromStr;

use mlua::{Lua, MultiValue, Table, Value};

use mosaico_core::ipc::{Command, DesktopSnapshot};
use mosaico_core::layout::LayoutKind;
use mosaico_core::tiler::Order;
use mosaico_core::{Action, MonitorId, WindowId};

use super::{command, failure, field, refused, success, to_lua};
use crate::convert;
use crate::runtime::State;

pub(super) fn install(lua: &Lua, mosaico: &Table, state: &State) -> mlua::Result<()> {
    let s = state.clone();
    mosaico.set(
        "state",
        lua.create_function(move |lua, ()| match desktop(&s) {
            Ok(desktop) => success(to_lua(lua, &convert::state(&desktop))?),
            Err(message) => failure(lua, message),
        })?,
    )?;

    let s = state.clone();
    mosaico.set(
        "windows",
        lua.create_function(move |lua, filter: Option<Table>| {
            let desktop = match desktop(&s) {
                Ok(desktop) => desktop,
                Err(message) => return failure(lua, message),
            };
            let state = convert::state(&desktop);
            let mut found = Vec::new();
            for window in state["windows"].as_array().into_iter().flatten() {
                if keep(window, filter.as_ref())? {
                    found.push(window.clone());
                }
            }
            success(to_lua(lua, &serde_json::Value::Array(found))?)
        })?,
    )?;

    let s = state.clone();
    mosaico.set(
        "action",
        lua.create_function(move |lua, name: String| match Action::from_str(&name) {
            Ok(action) => command(lua, &s, &Command::Action { action }),
            Err(_) => failure(lua, format!("unknown action `{name}`")),
        })?,
    )?;

    order(lua, mosaico, state, "focus", |(w, _, _): Args| {
        Ok(Order::Focus { window: window(w)? })
    })?;
    order(lua, mosaico, state, "send", |(w, ws, _): Args| {
        Ok(Order::Send {
            window: window(w)?,
            workspace: workspace(ws)?,
        })
    })?;
    order(lua, mosaico, state, "float", |(w, on, _): Args| {
        Ok(Order::Float {
            window: window(w)?,
            floating: on.as_boolean().unwrap_or(true),
        })
    })?;
    order(lua, mosaico, state, "close", |(w, _, _): Args| {
        Ok(Order::Close { window: window(w)? })
    })?;
    order(lua, mosaico, state, "show", |(m, ws, _): Args| {
        Ok(Order::Show {
            monitor: monitor(m)?,
            workspace: workspace(ws)?,
        })
    })?;
    order(lua, mosaico, state, "layout", |(m, ws, name): Args| {
        Ok(Order::Layout {
            monitor: monitor(m)?,
            workspace: workspace(ws)?,
            layout: layout(&name)?,
        })
    })?;
    order(lua, mosaico, state, "monocle", |(m, on, _): Args| {
        Ok(Order::Monocle {
            monitor: monitor(m)?,
            on: on.as_boolean().unwrap_or(true),
        })
    })?;
    Ok(())
}

/// The arguments of an order function, as the script passed them.
type Args = (Value, Value, Value);

/// Sets `mosaico.<name>` to a function that builds an order from its
/// arguments with `build` and sends it. Arguments that make no order
/// return `nil` and why.
fn order(
    lua: &Lua,
    mosaico: &Table,
    state: &State,
    name: &str,
    build: impl Fn(Args) -> Result<Order, String> + 'static,
) -> mlua::Result<()> {
    let s = state.clone();
    let function = lua.create_function(move |lua, args: Args| -> mlua::Result<MultiValue> {
        match build(args) {
            Ok(order) => command(lua, &s, &Command::Order { order }),
            Err(message) => failure(lua, message),
        }
    })?;
    mosaico.set(name, function)
}

fn desktop(state: &State) -> Result<DesktopSnapshot, String> {
    let daemon = state.borrow().env.daemon.clone();
    refused(daemon.send(&Command::State))?
        .data::<DesktopSnapshot>()
        .ok_or_else(|| "the daemon sent no desktop".into())
}

fn id(value: &Value, what: &str) -> Result<u64, String> {
    match value {
        Value::Integer(n) => u64::try_from(*n).map_err(|_| format!("{n} is not a {what} id")),
        _ => Err(format!("expected a {what} id")),
    }
}

fn window(value: Value) -> Result<WindowId, String> {
    id(&value, "window").map(WindowId::from_raw)
}

fn monitor(value: Value) -> Result<MonitorId, String> {
    id(&value, "monitor").map(MonitorId::from_raw)
}

/// A workspace numbered from 1, as scripts count them, as the daemon's
/// index from 0.
fn workspace(value: Value) -> Result<usize, String> {
    match value {
        Value::Integer(n) if n >= 1 => usize::try_from(n - 1).map_err(|e| e.to_string()),
        _ => Err("expected a workspace number, counted from 1".into()),
    }
}

fn layout(value: &Value) -> Result<LayoutKind, String> {
    let name = value
        .as_string()
        .map(|s| s.to_string_lossy())
        .ok_or("expected a layout name")?;
    serde_json::from_value(serde_json::Value::String(name.clone()))
        .map_err(|_| format!("unknown layout `{name}`; use bsp, vertical-stack or three-column"))
}

/// Whether `window` passes `filter`: equal `class`, `monitor`,
/// `workspace` and `floating`, and a `title` that contains the one given.
fn keep(window: &serde_json::Value, filter: Option<&Table>) -> mlua::Result<bool> {
    if let Some(class) = field::<String>(filter, "class")?
        && window["class"] != class.as_str()
    {
        return Ok(false);
    }
    if let Some(title) = field::<String>(filter, "title")?
        && !window["title"].as_str().is_some_and(|t| t.contains(&title))
    {
        return Ok(false);
    }
    for key in ["monitor", "workspace"] {
        if let Some(wanted) = field::<i64>(filter, key)?
            && window[key].as_i64() != Some(wanted)
        {
            return Ok(false);
        }
    }
    if let Some(floating) = field::<bool>(filter, "floating")?
        && window["floating"].as_bool() != Some(floating)
    {
        return Ok(false);
    }
    Ok(true)
}
