use std::process::Command;

/// Cleans a raw SSID, returning `None` when no usable text remains.
///
/// Trims surrounding whitespace and quotes (some frontends quote SSIDs) and
/// drops control characters, including stray NUL bytes.
pub(super) fn sanitize_ssid(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed);
    let cleaned: String = unquoted.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

/// Runs a helper command and parses its stdout with `parse`.
///
/// Command failures, empty output and invalid UTF-8 are handled gracefully:
/// stdout is decoded lossily and the parser decides whether it is usable.
fn run_ssid_command<F>(program: &str, args: &[&str], parse: F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse(&String::from_utf8_lossy(&output.stdout))
}

/// Extracts the first usable line from `iwgetid -r <iface>` output.
pub(super) fn parse_iwgetid_output(output: &str) -> Option<String> {
    output.lines().find_map(sanitize_ssid)
}

/// Extracts the `SSID:` field from `iw dev <iface> link` output.
pub(super) fn parse_iw_link_ssid(output: &str) -> Option<String> {
    for line in output.lines() {
        let Some(value) = line.trim().strip_prefix("SSID:") else {
            continue;
        };
        if let Some(ssid) = sanitize_ssid(value) {
            return Some(ssid);
        }
    }
    None
}

/// Extracts the active SSID from `nmcli -t -f active,ssid dev wifi` output.
pub(super) fn parse_nmcli_active_ssid(output: &str) -> Option<String> {
    for line in output.lines() {
        let mut fields = line.splitn(2, ':');
        let Some(active) = fields.next() else {
            continue;
        };
        if active.trim() != "yes" {
            continue;
        }
        let Some(ssid) = fields.next() else {
            continue;
        };
        // Terse nmcli output escapes colons inside SSIDs as `\:`.
        if let Some(ssid) = sanitize_ssid(&ssid.replace("\\:", ":")) {
            return Some(ssid);
        }
    }
    None
}

/// Resolves the SSID of the active Wi-Fi interface.
///
/// # Fallback order
/// 1. `iwgetid -r <iface>` — raw SSID, scoped to the detected interface.
/// 2. `iw dev <iface> link` — parses the `SSID:` line of the associated link.
/// 3. `nmcli -t -f active,ssid dev wifi` — NetworkManager's active Wi-Fi
///    network, used only when neither interface-scoped command works.
pub(super) fn wifi_ssid(iface: &str) -> Option<String> {
    if let Some(ssid) = run_ssid_command("iwgetid", &["-r", iface], parse_iwgetid_output) {
        return Some(ssid);
    }
    if let Some(ssid) = run_ssid_command("iw", &["dev", iface, "link"], parse_iw_link_ssid) {
        return Some(ssid);
    }
    run_ssid_command(
        "nmcli",
        &["-t", "-f", "active,ssid", "dev", "wifi"],
        parse_nmcli_active_ssid,
    )
}
