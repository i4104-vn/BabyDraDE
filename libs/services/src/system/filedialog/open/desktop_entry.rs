use std::path::Path;

/// Splits an `Exec=` value the way the Desktop Entry spec requires: quotes
/// group whitespace and backslash escapes the next character.
fn split_exec(exec: &str) -> Vec<String> {
    let (mut parts, mut current, mut quote, mut escaped) = (Vec::new(), String::new(), None, false);
    for ch in exec.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' | '\'' if quote == Some(ch) => quote = None,
            '"' | '\'' if quote.is_none() => quote = Some(ch),
            ch if ch.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            ch => current.push(ch),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

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
    let parts = split_exec(exec);
    let binary = parts.first()?.clone();
    let path_str = path.to_string_lossy();
    let mut args = Vec::new();
    let mut substituted = false;
    for part in parts.iter().skip(1) {
        let value = match part.as_str() {
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
