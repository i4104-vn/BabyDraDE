//! Subcommand dispatch for screenshot editor and clipboard picker processes.

/// Handles helper subcommands. Returns `true` when one was handled.
pub(crate) fn dispatch() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--screenshot-editor" {
        crate::shell::screenshot::open_editor(args[2].clone());
        return true;
    }
    if args.len() == 2 && args[1] == "--clipboard-picker" {
        crate::shell::clipboard::open_picker();
        return true;
    }
    false
}
