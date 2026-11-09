//! TopBar-specific CSS styling modularized by topic

pub const TOPBAR_CSS: &str = concat!(
    include_str!("base.css"),
    "\n",
    include_str!("left.css"),
    "\n",
    include_str!("island.css"),
    "\n",
    include_str!("right.css"),
);
