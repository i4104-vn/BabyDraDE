//! Pure parsing tests for network discovery helpers.
//!
//! These tests use only literal strings and temporary directories: no real
//! hardware or network commands are involved.

use std::fs;

use super::{
    is_virtual_interface, is_wifi_interface, parse_iw_link_ssid, parse_iwgetid_output,
    parse_nmcli_active_ssid, sanitize_ssid,
};

#[test]
fn iw_link_output_yields_the_associated_ssid() {
    let output = "Connected to 12:34:56:78:9a:bc (on wlp2s0)\n\
                  \tSSID: My Home Network\n\
                  \tfreq: 2412\n\
                  \tsignal: -48 dBm\n";
    assert_eq!(parse_iw_link_ssid(output), Some("My Home Network".into()));
}

#[test]
fn iw_link_output_without_ssid_is_ignored() {
    assert_eq!(parse_iw_link_ssid("Not connected.\n"), None);
    assert_eq!(parse_iw_link_ssid(""), None);
    assert_eq!(parse_iw_link_ssid("command failed: No such device\n"), None);
}

#[test]
fn iwgetid_output_skips_blank_lines() {
    let output = "\n\nHome-5G\n";
    assert_eq!(parse_iwgetid_output(output), Some("Home-5G".into()));
    assert_eq!(parse_iwgetid_output("\n"), None);
}

#[test]
fn nmcli_only_uses_the_active_row() {
    let output = "no:Neighbor\nyes:Home Network\nno:Guest\n";
    assert_eq!(parse_nmcli_active_ssid(output), Some("Home Network".into()));
}

#[test]
fn nmcli_handles_colons_and_escaped_colons_in_ssids() {
    let output = "no:Other\nyes:Cafe\\: Wifi:5G\n";
    assert_eq!(
        parse_nmcli_active_ssid(output),
        Some("Cafe: Wifi:5G".into())
    );
    assert_eq!(parse_nmcli_active_ssid("no:Only inactive\n"), None);
}

#[test]
fn sanitize_ssid_cleans_quotes_and_control_characters() {
    assert_eq!(sanitize_ssid("  \"My Wi-Fi\"\n"), Some("My Wi-Fi".into()));
    assert_eq!(sanitize_ssid("Home\0\nNet"), Some("HomeNet".into()));
    assert_eq!(sanitize_ssid("   "), None);
    assert_eq!(sanitize_ssid("\n\t\0"), None);
}

#[test]
fn virtual_interfaces_are_skipped() {
    for name in [
        "lo",
        "docker0",
        "vetha1b2c3",
        "virbr0",
        "br-123abc",
        "cni0",
        "flannel.1",
        "dummy0",
    ] {
        assert!(is_virtual_interface(name), "{name} should be skipped");
    }
    for name in ["eth0", "enp3s0", "wlan0", "wlp2s0"] {
        assert!(!is_virtual_interface(name), "{name} should be kept");
    }
}

#[test]
fn wifi_detection_checks_sysfs_markers_with_name_fallback() {
    let dir = tempfile::tempdir().expect("temporary sysfs directory");

    assert!(!is_wifi_interface(dir.path(), "eth0"));
    assert!(is_wifi_interface(dir.path(), "wlp2s0"));

    fs::write(dir.path().join("wireless"), "").expect("wireless marker");
    assert!(is_wifi_interface(dir.path(), "renamed0"));

    let other = tempfile::tempdir().expect("second temporary sysfs directory");
    fs::write(other.path().join("phy80211"), "").expect("phy80211 marker");
    assert!(is_wifi_interface(other.path(), "renamed0"));
}
