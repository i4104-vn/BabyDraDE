use super::*;

#[test]
fn read_locale_recovers_from_poisoned_lock() {
    let lock = RwLock::new("vi".to_string());

    // Poison the lock by panicking while holding a write guard.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = lock.write().expect("acquire locale lock");
        panic!("poison locale lock");
    }));
    assert!(lock.is_poisoned());

    // read_locale must not panic and must return a usable locale code.
    let locale = read_locale(&lock);
    assert!(locale == "en" || locale == "vi");
}

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
