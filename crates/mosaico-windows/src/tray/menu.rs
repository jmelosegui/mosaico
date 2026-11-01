//! The icon's menu.

use mosaico_core::tray::{TrayCommand, TrayItem};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, HMENU, MF_SEPARATOR, MF_STRING, PostMessageW,
    SetForegroundWindow, SetMenuDefaultItem, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
    WM_NULL,
};
use windows::core::PCWSTR;

/// Shows `items` as a menu at `x`, `y`, waits until the user picks an
/// entry or dismisses it, and returns the command picked.
pub(super) fn pick(hwnd: HWND, items: &[TrayItem], x: i32, y: i32) -> Option<TrayCommand> {
    // SAFETY: CreatePopupMenu makes an empty menu, destroyed below.
    let menu = match unsafe { CreatePopupMenu() } {
        Ok(menu) => menu,
        Err(e) => {
            mosaico_core::log_warn!("Could not open the tray menu: {e}");
            return None;
        }
    };
    let commands = fill(menu, items);
    // SAFETY: The window is the tray window of this thread. It has to be
    // in front for the menu to close when the user clicks elsewhere, and
    // the empty message after it lets the menu close properly.
    let chosen = unsafe {
        let _ = SetForegroundWindow(hwnd);
        let chosen = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            x,
            y,
            None,
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        chosen
    };
    command_for(&commands, chosen.0)
}

/// Appends `items` to `menu` and returns the commands in the order of
/// their ids, which start at 1 since 0 means nothing was picked.
fn fill(menu: HMENU, items: &[TrayItem]) -> Vec<TrayCommand> {
    let mut commands = Vec::new();
    for item in items {
        match item {
            TrayItem::Command { command, label } => {
                commands.push(*command);
                let id = commands.len();
                let text: Vec<u16> = label.encode_utf16().chain([0]).collect();
                // SAFETY: The label is a nul-terminated string that lives
                // through the call; the menu copies it.
                unsafe {
                    let _ = AppendMenuW(menu, MF_STRING, id, PCWSTR(text.as_ptr()));
                    if *command == TrayCommand::DEFAULT {
                        let _ = SetMenuDefaultItem(menu, id as u32, 0);
                    }
                }
            }
            TrayItem::Separator => {
                // SAFETY: A separator has no id and no text.
                unsafe {
                    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                }
            }
        }
    }
    commands
}

/// The command of menu id `id`, or `None` for 0, which means the menu
/// was dismissed.
fn command_for(commands: &[TrayCommand], id: i32) -> Option<TrayCommand> {
    let index = usize::try_from(id).ok()?.checked_sub(1)?;
    commands.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_count_from_one_and_zero_means_dismissed() {
        // Arrange
        let commands = [TrayCommand::OpenSettings, TrayCommand::Quit];

        // Act / Assert
        assert_eq!(command_for(&commands, 0), None);
        assert_eq!(command_for(&commands, 1), Some(TrayCommand::OpenSettings));
        assert_eq!(command_for(&commands, 2), Some(TrayCommand::Quit));
        assert_eq!(command_for(&commands, 3), None);
        assert_eq!(command_for(&commands, -1), None);
    }
}
