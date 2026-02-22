use mosaico_core::config;

/// ANSI escape helpers for doctor output.
const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";
const FIXED: &str = "\x1b[36m[fixed]\x1b[0m";

pub fn execute() {
    super::banner::print_logo();
    println!();
    check_config_dir();
    check_config_file();
    check_keybindings_file();
    check_keybinding_keys();
    check_rules_file();
    check_daemon();
    check_monitors();
    println!();
}

fn check_config_dir() {
    match config::config_dir() {
        Some(dir) if dir.is_dir() => {
            println!("  {OK} Config directory exists ({})", dir.display());
        }
        Some(dir) => match std::fs::create_dir_all(&dir) {
            Ok(()) => {
                println!("  {FIXED} Created config directory ({})", dir.display());
            }
            Err(e) => {
                println!("  {FAIL} Config directory missing and could not create it: {e}");
            }
        },
        None => {
            println!("  {FAIL} Could not determine home directory");
        }
    }
}

fn check_config_file() {
    let Some(path) = config::config_path() else {
        println!("  {FAIL} Could not determine config path");
        return;
    };
    if !path.exists() {
        println!("  {WARN} config.toml not found (using defaults)");
        return;
    }
    match config::try_load() {
        Ok(_) => println!("  {OK} config.toml is valid"),
        Err(e) => println!("  {FAIL} config.toml: {e}"),
    }
}

fn check_keybindings_file() {
    let Some(path) = config::keybindings_path() else {
        println!("  {FAIL} Could not determine keybindings path");
        return;
    };
    if !path.exists() {
        println!("  {WARN} keybindings.toml not found (using defaults)");
        return;
    }
    match config::try_load_keybindings() {
        Ok(_) => println!("  {OK} keybindings.toml is valid"),
        Err(e) => println!("  {FAIL} keybindings.toml: {e}"),
    }
}

fn check_keybinding_keys() {
    let bindings = config::load_keybindings();
    let mut bad: Vec<String> = Vec::new();
    for kb in &bindings {
        if mosaico_windows::keys::vk_from_name(&kb.key).is_none() {
            bad.push(kb.key.clone());
        }
    }
    if bad.is_empty() {
        println!(
            "  {OK} All {} keybinding(s) resolve to valid key codes",
            bindings.len()
        );
    } else {
        println!(
            "  {FAIL} {} keybinding(s) have unknown keys: {}",
            bad.len(),
            bad.join(", ")
        );
    }
}

fn check_rules_file() {
    let Some(path) = config::rules_path() else {
        println!("  {FAIL} Could not determine rules path");
        return;
    };
    if !path.exists() {
        println!("  {WARN} rules.toml not found (using defaults)");
        return;
    }
    match config::try_load_rules() {
        Ok(_) => println!("  {OK} rules.toml is valid"),
        Err(e) => println!("  {FAIL} rules.toml: {e}"),
    }
}

fn check_daemon() {
    if mosaico_windows::ipc::is_daemon_running() {
        if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
            println!("  {OK} Daemon is running (PID: {pid})");
        } else {
            println!("  {OK} Daemon is running");
        }
        return;
    }
    if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
        if mosaico_windows::process::is_process_alive(pid) {
            println!("  {WARN} Process exists (PID: {pid}) but not responding");
        } else {
            let _ = mosaico_core::pid::remove_pid_file();
            println!("  {FIXED} Removed stale PID file (PID: {pid})");
        }
    } else {
        println!("  {WARN} Daemon is not running");
    }
}

fn check_monitors() {
    match mosaico_windows::monitor::enumerate_monitors() {
        Ok(monitors) if monitors.is_empty() => {
            println!("  {FAIL} No monitors detected");
        }
        Ok(monitors) => {
            println!("  {OK} {} monitor(s) detected", monitors.len());
            for (i, m) in monitors.iter().enumerate() {
                let wa = &m.work_area;
                println!(
                    "       Monitor {i}: {}x{} at ({}, {})",
                    wa.width, wa.height, wa.x, wa.y
                );
            }
        }
        Err(e) => {
            println!("  {FAIL} Could not enumerate monitors: {e}");
        }
    }
}
