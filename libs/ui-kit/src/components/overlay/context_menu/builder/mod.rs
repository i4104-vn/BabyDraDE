//! Fluent builder for declaratively constructing and presenting context menus.

use super::items::*;
use gtk4::prelude::*;
use gtk4::{Box, Button, Popover, PositionType};
use std::cell::RefCell;
use std::rc::Rc;

mod construct;
mod items;
mod models;
mod widget;

pub use models::{dismiss_all, set_visibility_hook, ContextMenu, ContextMenuBuilder};
