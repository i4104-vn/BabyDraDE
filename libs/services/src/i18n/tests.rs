use super::*;

#[test]
fn test_i18n_translation() {
    set_locale("en");
    assert_eq!(trans("screenshot.pen_tooltip"), "Pen");
    assert_eq!(trans("lock.status"), "Device is locked");

    set_locale("vi");
    assert_eq!(trans("screenshot.pen_tooltip"), "Bút vẽ");
    assert_eq!(trans("lock.status"), "Thiết bị đang khóa");

    // Fallback for non-existent key returns key itself
    assert_eq!(trans("non_existent_key_123"), "non_existent_key_123");
}
