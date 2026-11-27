//! Result construction for the launcher search overlay.
//!
//! Applications come from the already-loaded launcher list, files/folders come
//! from a small, bounded index that is built once at startup, and the web row
//! is synthesized from the raw query. Nothing here touches the filesystem on a
//! keystroke, so typing never stalls the render loop. The bounded filesystem
//! walk itself lives in `services::files::index`; this module only scores the
//! snapshot and maps matches into result rows.

pub(crate) use handler::{build_results, url_encode};
pub(crate) use models::FileIndex;

mod handler;

mod models;
