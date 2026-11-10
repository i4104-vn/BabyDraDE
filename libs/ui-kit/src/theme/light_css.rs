//! Light-mode styles with the canonical `babydra-core` theme layer applied last.

pub const LIGHT_CSS: &str = concat!(
    include_str!("../../assets/styles/babydrade/light.css"),
    "\n",
    include_str!("../../assets/styles/core/themes/babydra-default/css/light.css"),
    "\n",
    include_str!("../../assets/styles/core/themes/babydra-default/css/theme.css"),
);
