//! `xdg-open` replacement and standalone file chooser CLI hosted by the compositor binary.
//!
//! `install.sh` points `xdg-open` here, so apps keep calling it unchanged.
//! With a target we open it; with no target (or `--filedialog-chooser`) we show
//! the native GTK4 chooser.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::Application;
use services::filedialog::open::open_target;
use services::filedialog::{FileDialogMode, FileDialogOptions, FileFilter};
use services::i18n::trans;
use ui_kit::components::modals::file_chooser::show;

/// Handles invocations made through the `xdg-open` symlink or `--filedialog-chooser`.
///
/// # Returns
/// `true` when this process handled the subcommand.
pub(crate) fn dispatch(args: &[String]) -> bool {
    let bin_name = args
        .first()
        .and_then(|arg| Path::new(arg).file_name())
        .and_then(|name| name.to_str());

    if bin_name == Some("xdg-open") {
        handle_xdg_open_args(&args[1..]);
        return true;
    }

    if args.get(1).map(String::as_str) == Some("--filedialog-chooser") {
        let options = parse_cli_options(&args[2..]);
        let exit_code = run_chooser(options, false);
        std::process::exit(exit_code);
    }

    false
}

/// Processes arguments when invoked as `xdg-open`.
fn handle_xdg_open_args(args: &[String]) {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("Usage: xdg-open {{ file | URL }}");
        std::process::exit(0);
    }
    if args.iter().any(|arg| arg == "--version") {
        println!("BabyDra (xdg-open) 1.0.0");
        std::process::exit(0);
    }

    let targets: Vec<&String> = args.iter().filter(|t| !t.starts_with('-')).collect();
    if targets.is_empty() {
        let options = FileDialogOptions {
            title: trans("file_dialog.title_open_file"),
            ..FileDialogOptions::default()
        };
        let exit_code = run_chooser(options, true);
        std::process::exit(exit_code);
    }

    let mut failed = false;
    for target in targets {
        if !open_target(target) {
            eprintln!("xdg-open: cannot open {target}");
            failed = true;
        }
    }
    std::process::exit(i32::from(failed));
}

/// Runs the GTK4 file chooser application loop.
///
/// When `open_picked` is `true`, the chosen path is opened via [`open_target`];
/// otherwise all chosen paths are printed to stdout (one per line).
fn run_chooser(options: FileDialogOptions, open_picked: bool) -> i32 {
    let app = Application::builder()
        .application_id("org.babydra.filedialog")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let exit_code = Rc::new(Cell::new(1));
    let exit_code_cb = exit_code.clone();

    app.connect_activate(move |_| {
        ui_kit::init_theme();
        let exit_code = exit_code_cb.clone();
        show(None, options.clone(), move |paths| {
            let Some(paths) = paths.filter(|p| !p.is_empty()) else {
                exit_code.set(1);
                return;
            };
            if open_picked {
                let ok = open_target(&paths[0].to_string_lossy());
                exit_code.set(i32::from(!ok));
            } else {
                for path in &paths {
                    println!("{}", path.to_string_lossy());
                }
                exit_code.set(0);
            }
        });
    });

    let _ = app.run_with_args::<&str>(&[]);
    exit_code.get()
}

/// Parses CLI options for `--filedialog-chooser`.
fn parse_cli_options(args: &[String]) -> FileDialogOptions {
    let mut mode = FileDialogMode::OpenFile;
    let mut custom_title = None;
    let mut initial_folder = None;
    let mut initial_file = None;
    let mut filters = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--open-file" => mode = FileDialogMode::OpenFile,
            "-d" | "--open-folder" => mode = FileDialogMode::OpenFolder,
            "-m" | "--open-multiple" => mode = FileDialogMode::OpenMultipleFiles,
            "-s" | "--save-file" => mode = FileDialogMode::SaveFile,
            "-t" | "--title" if i + 1 < args.len() => {
                i += 1;
                custom_title = Some(args[i].clone());
            }
            "-p" | "--path" if i + 1 < args.len() => {
                i += 1;
                initial_folder = Some(PathBuf::from(&args[i]));
            }
            "-n" | "--name" if i + 1 < args.len() => {
                i += 1;
                initial_file = Some(args[i].clone());
            }
            "--filter" if i + 1 < args.len() => {
                i += 1;
                if let Some(filter) = FileFilter::parse(&args[i]) {
                    filters.push(filter);
                }
            }
            other if !other.starts_with('-') && initial_folder.is_none() => {
                let path = PathBuf::from(other);
                if path.is_dir() {
                    initial_folder = Some(path);
                } else if let Some(parent) = path.parent() {
                    initial_folder = Some(parent.to_path_buf());
                    initial_file = path.file_name().and_then(|n| n.to_str()).map(String::from);
                }
            }
            _ => {}
        }
        i += 1;
    }

    if filters.is_empty() && mode != FileDialogMode::OpenFolder {
        filters.push(FileFilter::all_files());
    }

    let title = custom_title.unwrap_or_else(|| match mode {
        FileDialogMode::OpenFolder => trans("file_dialog.title_open_folder"),
        FileDialogMode::OpenFile => trans("file_dialog.title_open_file"),
        FileDialogMode::OpenMultipleFiles => trans("file_dialog.title_open_files"),
        FileDialogMode::SaveFile => trans("file_dialog.title_save_file"),
    });

    FileDialogOptions {
        title,
        mode,
        initial_folder,
        initial_file,
        default_filter_index: (!filters.is_empty()).then_some(0),
        filters,
        ..FileDialogOptions::default()
    }
}
