use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let bin_dir = Path::new(&manifest_dir).join("bin");
    let _ = fs::create_dir_all(&bin_dir);

    // Tell cargo to rerun this build script if anything in bin/ changes
    println!("cargo:rerun-if-changed=bin");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("embedded_widgets.rs");

    let mut code = String::from("pub static EMBEDDED_WIDGETS: &[(&str, &[u8])] = &[\n");

    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name != "babydrade" && !file_name.starts_with('.') {
                    let path_str = path.to_str().unwrap().replace('\\', "/");
                    code.push_str(&format!(
                        "    (\"{}\", include_bytes!(\"{}\")),\n",
                        file_name, path_str
                    ));
                }
            }
        }
    }

    code.push_str("];\n");
    fs::write(dest_path, code).unwrap();
}

