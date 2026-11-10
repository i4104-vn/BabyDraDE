//! Dark-mode styles with the canonical `babydra-core` theme layer applied last.

pub const DARK_CSS: &str = concat!(
    include_str!("../../assets/styles/babydrade/dark.css"),
    "\n",
    include_str!("../../assets/styles/core/themes/babydra-default/css/dark.css"),
    "\n",
    include_str!("../../assets/styles/core/themes/babydra-default/css/theme.css"),
);
