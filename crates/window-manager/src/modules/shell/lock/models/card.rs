//! Lock screen primary authentication card state.

pub(crate) struct PrimaryCard {
    pub(crate) card_box: gtk4::Box,
    pub(crate) entry: gtk4::PasswordEntry,
    pub(crate) status_label: gtk4::Label,
}
