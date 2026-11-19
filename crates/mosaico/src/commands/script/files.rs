//! `mosaico script new`, `edit`, `list`, `add` and `open`: the scripts
//! folder, without the user having to find it.

use std::fs;
use std::path::Path;

use super::paths::{Folders, invalid_name, name_of};
use crate::commands::context::{Context, Exit, complain, say};

/// A new script starts from this.
const TEMPLATE: &str = include_str!("template.lua");
/// The API's type annotations, for editors with a Lua language server.
const ANNOTATIONS: &str = include_str!("mosaico.lua");
/// Points the Lua language server at the annotations.
const LUARC: &str = include_str!("luarc.json");

/// Writes `scripts\<name>.lua` from the template and opens it. Also
/// writes the editor support files when they are missing.
pub fn new(cx: &mut Context, name: &str) -> Exit {
    if let Some(message) = invalid_name(name) {
        complain!(cx, "Error: {message}");
        return Exit::Failure;
    }
    let folders = Folders::new(cx.config.dir());
    let path = folders.script(name);
    if path.exists() {
        complain!(
            cx,
            "Error: {} already exists. Open it with `mosaico script edit {name}`.",
            path.display()
        );
        return Exit::Failure;
    }
    let template = TEMPLATE.replace("NAME", name);
    if let Err(e) = write_new(&folders, &path, &template) {
        complain!(cx, "Error: could not write {}: {e}", path.display());
        return Exit::Failure;
    }
    say!(cx, "Created {}", path.display());
    open(cx, &path)
}

fn write_new(folders: &Folders, path: &Path, template: &str) -> std::io::Result<()> {
    fs::create_dir_all(&folders.scripts)?;
    fs::write(path, template)?;
    fs::create_dir_all(&folders.lib)?;
    let annotations = folders.lib.join("mosaico.lua");
    if !annotations.exists() {
        fs::write(annotations, ANNOTATIONS)?;
    }
    let luarc = folders.root.join(".luarc.json");
    if !luarc.exists() {
        fs::write(luarc, LUARC)?;
    }
    Ok(())
}

/// Opens the script called `name`.
pub fn edit(cx: &mut Context, name: &str) -> Exit {
    let path = Folders::new(cx.config.dir()).script(name);
    if !path.exists() {
        complain!(
            cx,
            "Error: there is no script called {name}. Create it with `mosaico script new {name}`."
        );
        return Exit::Failure;
    }
    open(cx, &path)
}

/// Prints the names of the scripts in `scripts\`.
pub fn list(cx: &mut Context) -> Exit {
    let folders = Folders::new(cx.config.dir());
    let mut names: Vec<String> = fs::read_dir(&folders.scripts)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "lua"))
        .map(|path| name_of(&path))
        .collect();
    names.sort();
    if names.is_empty() {
        say!(
            cx,
            "No scripts in {}. Create one with `mosaico script new <name>`.",
            folders.scripts.display()
        );
    }
    for name in names {
        say!(cx, "{name}");
    }
    Exit::Success
}

/// Copies the script at `source` into `scripts\`, under its own name.
pub fn add(cx: &mut Context, source: &Path) -> Exit {
    let name = name_of(source);
    if let Some(message) = invalid_name(&name) {
        complain!(cx, "Error: {message}");
        return Exit::Failure;
    }
    let folders = Folders::new(cx.config.dir());
    let target = folders.script(&name);
    if target.exists() {
        complain!(cx, "Error: there is a script called {name} already.");
        return Exit::Failure;
    }
    let copied = fs::create_dir_all(&folders.scripts).and_then(|()| fs::copy(source, &target));
    if let Err(e) = copied {
        complain!(cx, "Error: could not copy {}: {e}", source.display());
        return Exit::Failure;
    }
    say!(cx, "Added {name}. Run it with `mosaico script run {name}`.");
    Exit::Success
}

/// Shows the scripts folder in the file manager, creating it first.
pub fn open_folder(cx: &mut Context) -> Exit {
    let folders = Folders::new(cx.config.dir());
    if let Err(e) = fs::create_dir_all(&folders.scripts) {
        complain!(
            cx,
            "Error: could not create {}: {e}",
            folders.scripts.display()
        );
        return Exit::Failure;
    }
    open(cx, &folders.scripts)
}

fn open(cx: &mut Context, path: &Path) -> Exit {
    match cx.opener.open(path) {
        Ok(()) => Exit::Success,
        Err(e) => {
            complain!(cx, "Could not open {}: {e}", path.display());
            Exit::Failure
        }
    }
}
