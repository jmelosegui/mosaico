/// Manages automatic startup when Windows boots.
///
/// Delegates to [`mosaico_windows::autostart`] for registry operations.
pub fn enable() {
    match mosaico_windows::autostart::enable() {
        Ok(()) => println!("Autostart enabled."),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn disable() {
    match mosaico_windows::autostart::disable() {
        Ok(()) => println!("Autostart disabled."),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn status() {
    if mosaico_windows::autostart::is_enabled() {
        println!("Autostart is currently enabled.");
    } else {
        println!("Autostart is currently disabled.");
    }
}
