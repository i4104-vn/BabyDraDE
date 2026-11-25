//! Internal registration record for one island view.

use std::cell::RefCell;
use std::rc::Rc;

use super::feature::IslandFeature;
use super::handle::ViewState;

pub(crate) struct ViewRecord {
    pub(crate) feature: Rc<RefCell<dyn IslandFeature>>,
    pub(crate) container: gtk4::Box,
    pub(crate) state: Rc<ViewState>,
}
