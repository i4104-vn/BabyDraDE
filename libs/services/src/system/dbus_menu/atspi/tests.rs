use super::ids::{atspi_bus_name, classify_role, encode_atspi_source, state_has, AccessibleRole};

#[test]
fn source_encoding_round_trips() {
    let (service, path) = encode_atspi_source(":1.42", "/org/a11y/atspi/accessible/1");
    assert_eq!(atspi_bus_name(&service), Some(":1.42"));
    assert_eq!(path, "/org/a11y/atspi/accessible/1");
    assert_eq!(atspi_bus_name("org.example.App"), None);
}

#[test]
fn states_are_packed_into_words_of_32() {
    let states = [0x0100_0001u32, 1u32 << 10, 0];
    assert!(state_has(&states, 0));
    assert!(state_has(&states, 24));
    assert!(state_has(&states, 42));
    assert!(!state_has(&states, 1));
    assert!(!state_has(&states, 64));
}

#[test]
fn menu_roles_are_classified_before_generic_roles() {
    assert_eq!(classify_role("menu bar"), AccessibleRole::MenuBar);
    assert_eq!(classify_role("menuBar"), AccessibleRole::MenuBar);
    assert_eq!(classify_role("menu item"), AccessibleRole::MenuItem);
    assert_eq!(classify_role("check menu item"), AccessibleRole::MenuItem);
    assert_eq!(classify_role("separator"), AccessibleRole::Separator);
    assert_eq!(classify_role("menu"), AccessibleRole::Menu);
    assert_eq!(classify_role("window"), AccessibleRole::Other);
}
