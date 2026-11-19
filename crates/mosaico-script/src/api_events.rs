//! `mosaico.on`, `mosaico.wait`, `mosaico.sleep`, the timers and
//! `mosaico.stop`.

use std::str::FromStr;
use std::time::{Duration, Instant};

use mlua::{Function, Lua, LuaSerdeExt, MultiValue, Table, Value};

use mosaico_core::desktop_events::EventKind;

use super::{event_value, failure, success};
use crate::convert;
use crate::env::Next;
use crate::runtime::{STOP, State, Timer};

pub(super) fn install(lua: &Lua, mosaico: &Table, state: &State) -> mlua::Result<()> {
    let s = state.clone();
    mosaico.set(
        "on",
        lua.create_function(move |_, (name, handler): (String, Function)| {
            let kind = EventKind::from_str(&name).map_err(mlua::Error::external)?;
            let mut shared = s.borrow_mut();
            shared.handlers.push((kind, handler));
            // From here on, nothing that happens is missed. When the
            // daemon is not there yet, the loop subscribes later.
            shared.ensure_subscribed();
            Ok(())
        })?,
    )?;

    let s = state.clone();
    mosaico.set(
        "wait",
        lua.create_function(move |lua, args: MultiValue| wait(lua, &s, args))?,
    )?;

    let s = state.clone();
    mosaico.set(
        "sleep",
        lua.create_function(move |_, seconds: f64| {
            let deadline = after(&s, seconds)?;
            sleep_until(&s, deadline);
            Ok(())
        })?,
    )?;

    for (name, repeats) in [("after", false), ("every", true)] {
        let s = state.clone();
        let function = lua.create_function(move |_, (seconds, run): (f64, Function)| {
            let due = after(&s, seconds)?;
            let mut shared = s.borrow_mut();
            let id = shared.next_timer;
            shared.next_timer += 1;
            let every = repeats.then(|| Duration::from_secs_f64(seconds));
            shared.timers.push(Timer {
                id,
                due,
                every,
                run,
            });
            Ok(id)
        })?;
        mosaico.set(name, function)?;
    }

    let s = state.clone();
    mosaico.set(
        "cancel",
        lua.create_function(move |_, id: u64| {
            let mut shared = s.borrow_mut();
            let before = shared.timers.len();
            shared.timers.retain(|t| t.id != id);
            Ok(shared.timers.len() != before)
        })?,
    )?;

    let s = state.clone();
    mosaico.set(
        "stop",
        lua.create_function(move |_, code: Option<i32>| -> mlua::Result<()> {
            s.borrow_mut().stop = Some(code.unwrap_or(0));
            Err(mlua::Error::runtime(STOP))
        })?,
    )?;
    Ok(())
}

/// When `seconds` from now is.
fn after(state: &State, seconds: f64) -> mlua::Result<Instant> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(mlua::Error::runtime(
            "expected a number of seconds, 0 or more",
        ));
    }
    let now = state.borrow().env.clock.now();
    Ok(now + Duration::from_secs_f64(seconds))
}

/// Waits until `deadline`. Events that arrive meanwhile are kept for the
/// handlers, which run once the script is back in its loop.
fn sleep_until(state: &State, deadline: Instant) {
    loop {
        let next = {
            let mut shared = state.borrow_mut();
            if shared.inbox.is_none() {
                None
            } else {
                Some(shared.inbox.as_mut().map(|i| i.next(Some(deadline))))
            }
        };
        match next {
            Some(Some(Next::Event(event))) => state.borrow_mut().pending.push_back(event),
            Some(Some(Next::Closed)) => {
                state.borrow_mut().inbox = None;
                let clock = state.borrow().env.clock.clone();
                clock.sleep_until(deadline);
                return;
            }
            Some(_) => return,
            None => {
                let clock = state.borrow().env.clock.clone();
                clock.sleep_until(deadline);
                return;
            }
        }
    }
}

/// `mosaico.wait(kind [, filter] [, seconds])`: the next event of `kind`
/// whose fields equal those of `filter`, or `nil` and why when `seconds`
/// pass first or the daemon goes away. Other events are kept for the
/// handlers.
fn wait(lua: &Lua, state: &State, args: MultiValue) -> mlua::Result<MultiValue> {
    let mut args = args.into_iter();
    let name: String = match args.next() {
        Some(Value::String(s)) => s.to_str()?.to_string(),
        _ => return Err(mlua::Error::runtime("expected an event name")),
    };
    let kind = EventKind::from_str(&name).map_err(mlua::Error::external)?;
    let mut filter = serde_json::Value::Null;
    let mut seconds = None;
    for arg in args {
        match arg {
            Value::Table(t) => filter = lua.from_value(Value::Table(t))?,
            Value::Integer(n) => seconds = Some(n as f64),
            Value::Number(n) => seconds = Some(n),
            _ => {}
        }
    }
    let deadline = seconds.map(|s| after(state, s)).transpose()?;
    if !state.borrow_mut().ensure_subscribed() {
        return failure(lua, "mosaico is not running");
    }

    // An event that arrived while the script was busy counts too.
    let kept = {
        let mut shared = state.borrow_mut();
        let found = shared
            .pending
            .iter()
            .position(|e| e.kind() == kind && convert::matches(&convert::event(e), &filter));
        found.and_then(|i| shared.pending.remove(i))
    };
    if let Some(event) = kept {
        return success(event_value(lua, &event)?);
    }
    loop {
        let next = {
            let mut shared = state.borrow_mut();
            match shared.inbox.as_mut() {
                Some(inbox) => inbox.next(deadline),
                None => Next::Closed,
            }
        };
        match next {
            Next::Event(event) => {
                if event.kind() == kind && convert::matches(&convert::event(&event), &filter) {
                    return success(event_value(lua, &event)?);
                }
                state.borrow_mut().pending.push_back(event);
            }
            Next::TimedOut => return failure(lua, "timed out"),
            Next::Closed => {
                state.borrow_mut().inbox = None;
                return failure(lua, "mosaico stopped");
            }
        }
    }
}
