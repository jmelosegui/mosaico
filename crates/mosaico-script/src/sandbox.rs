//! What a script may not reach: a shell, native modules, or modules from
//! anywhere but the scripts' library folder.

use std::path::Path;

use mlua::{Lua, Table, Value};

/// Removes the functions that run a shell or load native code, and points
/// `require` at `lib`, or at nothing without one. `mosaico.exec` runs
/// programs instead, from an argument list and without a shell, and
/// `mosaico.stop` ends the script.
pub fn apply(lua: &Lua, lib: Option<&Path>) -> mlua::Result<()> {
    let globals = lua.globals();
    let os: Table = globals.get("os")?;
    os.set("execute", Value::Nil)?;
    os.set("exit", Value::Nil)?;
    let io: Table = globals.get("io")?;
    io.set("popen", Value::Nil)?;
    let package: Table = globals.get("package")?;
    package.set("loadlib", Value::Nil)?;
    package.set("cpath", "")?;
    let path = lib.map_or_else(String::new, |dir| {
        format!("{}/?.lua", dir.to_string_lossy().replace('\\', "/"))
    });
    package.set("path", path)?;
    // Only the Lua searchers: the preloaded modules and `path`.
    let searchers: Table = package.get("searchers")?;
    searchers.set(3, Value::Nil)?;
    searchers.set(4, Value::Nil)?;
    Ok(())
}
