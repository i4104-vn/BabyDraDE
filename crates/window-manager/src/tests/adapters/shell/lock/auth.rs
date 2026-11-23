use super::*;

#[test]
fn test_pam_symbols_or_graceful_fallback() {
    // Calling get_pam should never panic; on standard Linux it finds PAM, or returns None cleanly.
    let _pam = get_pam();
}
