//! `mosaico.launch`, `mosaico.exec`, `mosaico.log`, `mosaico.json`,
//! `mosaico.args` and `print`.

use std::time::Duration;

use mlua::{Lua, LuaSerdeExt, MultiValue, Table, Value, Variadic};
use mosaico_core::system::Visibility;

use super::{failure, success, to_lua};
use crate::runtime::State;

pub(super) fn install(
    lua: &Lua,
    mosaico: &Table,
    state: &State,
    args: &[(String, String)],
) -> mlua::Result<()> {
    let s = state.clone();
    mosaico.set(
        "launch",
        lua.create_function(move |lua, spec: Table| {
            let argv = argv(&spec)?;
            let Some((command, args)) = argv.split_first() else {
                return failure(lua, "expected the program to start");
            };
            let launcher = s.borrow().env.launcher.clone();
            match launcher.launch(command, args, Visibility::Shown) {
                Ok(()) => success(Value::Boolean(true)),
                Err(e) => failure(lua, e.to_string()),
            }
        })?,
    )?;

    let s = state.clone();
    mosaico.set(
        "exec",
        lua.create_function(move |lua, spec: Table| -> mlua::Result<MultiValue> {
            let argv = argv(&spec)?;
            if argv.is_empty() {
                return failure(lua, "expected the program to run");
            }
            let timeout: Option<f64> = spec.get("timeout")?;
            let timeout = timeout.filter(|t| *t > 0.0).map(Duration::from_secs_f64);
            let runner = s.borrow().env.runner.clone();
            match runner.run(&argv, timeout) {
                Ok(ran) => {
                    let result = lua.create_table()?;
                    result.set("ok", ran.code == Some(0))?;
                    result.set("code", ran.code)?;
                    result.set("stdout", ran.stdout)?;
                    result.set("stderr", ran.stderr)?;
                    result.set("timed_out", ran.timed_out)?;
                    success(Value::Table(result))
                }
                Err(e) => failure(lua, e),
            }
        })?,
    )?;

    let s = state.clone();
    mosaico.set(
        "log",
        lua.create_function(move |_, words: Variadic<String>| {
            let (level, text) = match words.as_slice() {
                [level, text] if is_level(level) => (level.as_str(), text.as_str()),
                [text, ..] => ("info", text.as_str()),
                [] => ("info", ""),
            };
            let err = s.borrow().env.err.clone();
            let _ = writeln!(err.borrow_mut(), "[{level}] {text}");
            Ok(())
        })?,
    )?;

    let s = state.clone();
    lua.globals().set(
        "print",
        lua.create_function(move |_, values: Variadic<Value>| {
            let texts: Vec<String> = values
                .iter()
                .map(|v| v.to_string().unwrap_or_else(|_| format!("{v:?}")))
                .collect();
            let out = s.borrow().env.out.clone();
            let _ = writeln!(out.borrow_mut(), "{}", texts.join("\t"));
            Ok(())
        })?,
    )?;

    let json = lua.create_table()?;
    json.set(
        "encode",
        lua.create_function(|lua, value: Value| {
            let value: serde_json::Value = lua.from_value(value)?;
            Ok(value.to_string())
        })?,
    )?;
    json.set(
        "decode",
        lua.create_function(|lua, text: String| match serde_json::from_str(&text) {
            Ok(value) => success(to_lua(lua, &value)?),
            Err(e) => failure(lua, e.to_string()),
        })?,
    )?;
    mosaico.set("json", json)?;

    let table = lua.create_table()?;
    for (key, value) in args {
        table.set(key.as_str(), typed(lua, value)?)?;
    }
    mosaico.set("args", table)?;
    Ok(())
}

fn is_level(word: &str) -> bool {
    matches!(word, "debug" | "info" | "warn" | "error")
}

/// The list part of `spec` as the program and its arguments.
fn argv(spec: &Table) -> mlua::Result<Vec<String>> {
    spec.sequence_values::<String>().collect()
}

/// An argument as the value it reads as: a number, `true` or `false`, or
/// the text itself.
fn typed(lua: &Lua, value: &str) -> mlua::Result<Value> {
    if let Ok(n) = value.parse::<i64>() {
        return Ok(Value::Integer(n));
    }
    if let Ok(n) = value.parse::<f64>() {
        return Ok(Value::Number(n));
    }
    Ok(match value {
        "true" => Value::Boolean(true),
        "false" => Value::Boolean(false),
        text => Value::String(lua.create_string(text)?),
    })
}
