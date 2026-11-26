//! Subcommand dispatch for the screenshot editor and the `xdg-open` shim.

/// Handles helper invocations. Returns `true` when one was handled.
pub(crate) fn dispatch() -> bool {
    let args: Vec<String> = std::env::args().collect();

    if crate::modules::shell::filedialog::dispatch(&args) {
        return true;
    }

    if args.len() == 3 && args[1] == "--screenshot-editor" {
        crate::modules::shell::screenshot::open_editor(args[2].clone());
        return true;
    }
    false
}
