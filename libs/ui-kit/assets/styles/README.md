# UI Kit styles

`core/shared/` mirrors `babydra-core/crates/babydra-ui-kit/src/styles/shared/`.
`core/themes/babydra-default/css/` mirrors the core default theme CSS.

`babydrade/` keeps selectors used by BabyDraDE widgets that are not defined in
core. These rules load before core styles, so shared component styling comes
from the core layer. Keep the include order in `src/theme/shared_css.rs` aligned
with `babydra-core/crates/babydra-ui-kit/src/ui/theme/mod.rs`.
