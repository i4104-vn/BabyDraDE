use std::path::Path;

/// Parses `Exec=` and `Terminal=`, substituting `%f`/`%u` fields with `path`/`uri`.
pub(super) fn parse_desktop_entry(
    content: &str,
    path: &Path,
    uri: &str,
) -> Option<(String, Vec<String>, bool)> {
    let exec = content
        .lines()
        .filter_map(|l| l.trim().strip_prefix("Exec="))
        .next_back()?
        .trim();
    let is_terminal = content
        .lines()
        .any(|l| l.trim().eq_ignore_ascii_case("Terminal=true"));
    let mut parts = exec.split_whitespace();
    let binary = parts.next()?.trim_matches('"').to_string();
    let path_str = path.to_string_lossy();
    let mut args = Vec::new();
    let mut substituted = false;
    for part in parts {
        let clean = part.trim_matches('"');
        let value = match clean {
            "%f" | "%F" => {
                substituted = true;
                path_str.as_ref()
            }
            "%u" | "%U" => {
                substituted = true;
                uri
            }
            other if !other.starts_with('%') => other,
            _ => continue,
        };
        args.push(value.to_string());
    }
    if !substituted {
        args.push(path_str.into_owned());
    }
    Some((binary, args, is_terminal))
}
