//! Pointer button event context.

pub(super) struct ButtonContext<'a, C> {
    pub(super) state: &'a mut C,
    pub(super) serial: smithay::utils::Serial,
    pub(super) button_code: u32,
    pub(super) time: u32,
}
