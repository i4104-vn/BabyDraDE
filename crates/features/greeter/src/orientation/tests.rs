//! Tests for the orientation helper functions.

use super::{auto::rotate_wl, config::user_home};
use wayland_client::protocol::wl_output::Transform;

#[test]
fn rotate_wl_keeps_mirrored_family_while_turning() {
    assert_eq!(rotate_wl(Transform::Normal, 1), Transform::_90);
    assert_eq!(rotate_wl(Transform::Normal, 4), Transform::Normal);
    assert_eq!(rotate_wl(Transform::Flipped90, 1), Transform::Flipped180);
}

#[test]
fn user_home_reads_the_sixth_passwd_field() {
    let passwd = "root:x:0:0:root:/root:/bin/bash\nuser:x:1000:1000::/home/user:/bin/zsh\n";
    assert_eq!(
        user_home("user", passwd),
        Some(std::path::PathBuf::from("/home/user"))
    );
    assert_eq!(user_home("missing", passwd), None);
}
