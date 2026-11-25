//! Subcommand dispatch for helper processes such as the screenshot editor.

/// Handles helper subcommands. Returns `true` when one was handled.
pub(crate) fn dispatch() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--screenshot-editor" {
        crate::modules::shell::screenshot::open_editor(args[2].clone());
        return true;
    }
    false
}
