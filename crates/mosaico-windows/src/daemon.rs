use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use mosaico_core::ipc::{Command, Response};
use mosaico_core::{Action, BspLayout, WindowResult, config, pid};

use crate::bar_manager::BarManager;
use crate::config_watcher::{self, ConfigReload};
use crate::dpi;
use crate::event_loop;
use crate::ipc::PipeServer;
use crate::monitor;
use crate::tiling::TilingManager;

/// Runs the Mosaico daemon.
///
/// Starts background threads for the Win32 event loop (which also
/// handles global hotkeys) and the IPC listener. The main thread
/// manages the tiling workspace.
pub fn run() -> WindowResult<()> {
    dpi::enable_dpi_awareness();
    pid::write_pid_file()?;
    eprintln!("Mosaico daemon started.");

    let result = daemon_loop();

    let _ = pid::remove_pid_file();

    result
}

/// Internal message type for the main daemon thread.
enum DaemonMsg {
    /// A window event from the event loop.
    Event(mosaico_core::WindowEvent),
    /// A user action from hotkeys or IPC.
    Action(Action),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
    /// A validated config reload from the file watcher.
    Reload(Box<ConfigReload>),
    /// 1-second tick for refreshing bar system widgets.
    Tick,
}

/// Sends a response back to the IPC thread for the connected client.
type ResponseSender = mpsc::Sender<Response>;

/// The inner daemon loop, separated so cleanup always runs in `run()`.
fn daemon_loop() -> WindowResult<()> {
    let config = config::load();
    mosaico_core::log::init(&config.logging);

    let keybindings = config::load_keybindings();
    let rules = config::load_rules();

    mosaico_core::log_info!("Daemon started (PID: {})", std::process::id());
    mosaico_core::log_info!(
        "Config: layout(gap={}, ratio={}), borders(width={}), log_level={}",
        config.layout.gap,
        config.layout.ratio,
        config.borders.width,
        config.logging.level
    );

    let layout = BspLayout {
        gap: config.layout.gap,
        ratio: config.layout.ratio,
    };

    let mut current_theme = config.theme.resolve();

    let bar_config = config::load_bar();
    let monitor_rects: Vec<_> = monitor::enumerate_monitors()?
        .iter()
        .map(|m| m.work_area)
        .collect();
    let mut bar_mgr = BarManager::new(bar_config, monitor_rects, current_theme);

    let (tx, rx) = mpsc::channel::<DaemonMsg>();

    let mut manager = TilingManager::new(layout, rules, config.borders)?;
    let bar_height = bar_mgr.bar_height();
    if bar_height > 0 {
        // Retile with bar-adjusted work areas so borders match final positions.
        manager.adjust_work_areas_for_bar(bar_height, bar_mgr.bar_monitor_indices());
    }
    manager.refresh_border();
    bar_mgr.update(&manager.bar_states());
    mosaico_core::log_info!("Managing {} windows", manager.window_count());

    // Start the Win32 event loop + hotkeys on its own thread.
    let event_tx = tx.clone();
    let action_tx = tx.clone();
    let (event_channel_tx, event_channel_rx) = mpsc::channel();
    let (action_channel_tx, action_channel_rx) = mpsc::channel();
    let event_loop = event_loop::start(event_channel_tx, action_channel_tx, keybindings)?;

    // Bridge: forward window events into the unified channel.
    let event_bridge = thread::spawn(move || {
        for event in event_channel_rx {
            if event_tx.send(DaemonMsg::Event(event)).is_err() {
                break;
            }
        }
    });

    // Bridge: forward hotkey actions into the unified channel.
    let action_bridge = thread::spawn(move || {
        for action in action_channel_rx {
            if action_tx.send(DaemonMsg::Action(action)).is_err() {
                break;
            }
        }
    });

    // Start the IPC listener on its own thread.
    let ipc_tx = tx.clone();
    let ipc_thread = thread::spawn(move || ipc_loop(ipc_tx));

    // Start the config file watcher on its own thread.
    let (reload_tx, reload_rx) = mpsc::channel::<ConfigReload>();
    let watcher_stop = Arc::new(AtomicBool::new(false));
    let watcher_stop_flag = watcher_stop.clone();
    let watcher_thread = thread::spawn(move || config_watcher::watch(reload_tx, watcher_stop_flag));

    // Bridge: forward config reloads into the unified channel.
    let reload_bridge_tx = tx.clone();
    let reload_bridge = thread::spawn(move || {
        for reload in reload_rx {
            if reload_bridge_tx
                .send(DaemonMsg::Reload(Box::new(reload)))
                .is_err()
            {
                break;
            }
        }
    });

    // 1-second tick for bar system widget refresh (clock, RAM).
    let tick_tx = tx.clone();
    let tick_stop = watcher_stop.clone();
    let tick_thread = thread::spawn(move || {
        while !tick_stop.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_secs(1));
            if tick_tx.send(DaemonMsg::Tick).is_err() {
                break;
            }
        }
    });

    // Main processing loop — blocks until a message arrives.
    while let Ok(msg) = rx.recv() {
        match msg {
            DaemonMsg::Event(event) => {
                manager.handle_event(&event);
                bar_mgr.update(&manager.bar_states());
            }
            DaemonMsg::Action(action) => {
                manager.handle_action(&action);
                bar_mgr.update(&manager.bar_states());
            }
            DaemonMsg::Command(Command::Stop, reply_tx) => {
                let response = Response::ok_with_message("Daemon stopping");
                let _ = reply_tx.send(response);
                mosaico_core::log_info!("Stop command received, shutting down");
                break;
            }
            DaemonMsg::Command(Command::Status, reply_tx) => {
                let msg = format!(
                    "Daemon is running, managing {} windows",
                    manager.window_count()
                );
                let response = Response::ok_with_message(msg);
                let _ = reply_tx.send(response);
            }
            DaemonMsg::Command(Command::Action { action }, reply_tx) => {
                manager.handle_action(&action);
                bar_mgr.update(&manager.bar_states());
                let response = Response::ok();
                let _ = reply_tx.send(response);
            }
            DaemonMsg::Reload(reload) => match *reload {
                ConfigReload::Config(cfg) => {
                    current_theme = cfg.theme.resolve();
                    manager.reload_config(&cfg);
                    // Theme may have changed — re-resolve bar colors.
                    bar_mgr.resolve_colors(current_theme);
                    bar_mgr.update(&manager.bar_states());
                }
                ConfigReload::Rules(rules) => {
                    manager.reload_rules(rules);
                }
                ConfigReload::Bar(bar_cfg) => {
                    let new_height = bar_mgr.reload(*bar_cfg);
                    bar_mgr.resolve_colors(current_theme);
                    let indices = bar_mgr.bar_monitor_indices().to_vec();
                    manager.reset_and_adjust_work_areas(new_height, &indices);
                    bar_mgr.update(&manager.bar_states());
                }
            },
            DaemonMsg::Tick => {
                bar_mgr.update(&manager.bar_states());
            }
        }
    }

    manager.restore_all_windows();
    bar_mgr.hide_all();
    event_loop.stop();
    watcher_stop.store(true, Ordering::Relaxed);
    drop(tx);
    let _ = event_bridge.join();
    let _ = action_bridge.join();
    let _ = watcher_thread.join();
    let _ = reload_bridge.join();
    let _ = tick_thread.join();
    let _ = ipc_thread.join();

    Ok(())
}

/// Accepts IPC connections in a loop and forwards commands to the
/// main daemon thread. Runs on a dedicated thread.
fn ipc_loop(tx: mpsc::Sender<DaemonMsg>) {
    loop {
        let server = match PipeServer::create() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create pipe: {e}");
                return;
            }
        };

        let command = match server.accept_command() {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error reading command: {e}");
                continue;
            }
        };

        let (reply_tx, reply_rx) = mpsc::channel();
        let is_stop = matches!(command, Command::Stop);

        if tx.send(DaemonMsg::Command(command, reply_tx)).is_err() {
            return;
        }

        if let Ok(response) = reply_rx.recv() {
            let _ = server.send_response(&response);
        }

        if is_stop {
            return;
        }
    }
}
