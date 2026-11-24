//! Minimal reader for the Xcursor file format used by GTK/X11 cursor themes.
//!
//! This intentionally avoids any GTK dependency: the format is just a header,
//! a table of image sizes, and ARGB pixel chunks, all of which we parse here
//! (layout follows `X11/Xcursor/Xcursor.h`):
//!
//! ```text
//! file header:  magic "Xcur" | header_len | version | ntoc
//! toc entry:    type | subtype (= nominal size) | file offset
//! image chunk:  header_len | type | subtype | version
//!               width | height | xhot | yhot | delay
//!               ARGB pixels
//! ```

use super::{premultiply, CursorImage};

const HEADER_MAGIC: &[u8; 4] = b"Xcur";
const IMAGE_TYPE: u32 = 0xfffd_0002;
const HEADER_LEN: usize = 16;
const TOC_ENTRY_LEN: usize = 12;
const IMAGE_HEADER_LEN: usize = 36;
// Animated cursors can list one entry per frame per size (Bibata's `wait`
// has 756). Keep a bound, but high enough for real themes.
const MAX_TOC_ENTRIES: usize = 4096;
const MAX_DIMENSION: u32 = 512;

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

/// Parses an Xcursor file and returns the image whose nominal size is closest
/// to `preferred_size`.
pub fn parse_xcursor(data: &[u8], preferred_size: u32) -> Option<CursorImage> {
    if data.len() < HEADER_LEN || &data[..4] != HEADER_MAGIC {
        return None;
    }

    let header_len = read_u32(data, 4)? as usize;
    if header_len < HEADER_LEN || header_len > data.len() {
        return None;
    }
    // XCURSOR_FILE_VERSION packs major/minor as (major << 16) | minor,
    // so version 1.0 is stored as 0x00010000.
    let version = read_u32(data, 8)?;
    if version >> 16 != 1 {
        return None;
    }
    let ntoc = read_u32(data, 12)? as usize;
    if ntoc == 0 || ntoc > MAX_TOC_ENTRIES {
        return None;
    }
    let toc_end = header_len.checked_add(ntoc.checked_mul(TOC_ENTRY_LEN)?)?;
    if toc_end > data.len() {
        return None;
    }

    let mut best: Option<(i64, usize)> = None;
    for i in 0..ntoc {
        let entry = header_len + i * TOC_ENTRY_LEN;
        let kind = read_u32(data, entry)?;
        let subtype = read_u32(data, entry + 4)?;
        let offset = read_u32(data, entry + 8)? as usize;

        if kind != IMAGE_TYPE {
            continue;
        }

        let distance = (subtype as i64 - preferred_size as i64).abs();
        if best.is_none_or(|(best_distance, _)| distance < best_distance) {
            best = Some((distance, offset));
        }
    }

    let (_, offset) = best?;
    parse_chunk(data, offset)
}

fn parse_chunk(data: &[u8], offset: usize) -> Option<CursorImage> {
    let chunk_header_len = read_u32(data, offset)? as usize;
    if chunk_header_len < IMAGE_HEADER_LEN || read_u32(data, offset + 4)? != IMAGE_TYPE {
        return None;
    }
    if read_u32(data, offset + 12)? != 1 {
        return None;
    }

    let width = read_u32(data, offset + 16)?;
    let height = read_u32(data, offset + 20)?;
    let xhot = read_u32(data, offset + 24)?;
    let yhot = read_u32(data, offset + 28)?;

    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || xhot >= width
        || yhot >= height
    {
        return None;
    }

    let count = width as usize * height as usize;
    let pixels_start = offset.checked_add(chunk_header_len)?;
    let pixels_end = pixels_start.checked_add(count.checked_mul(4)?)?;
    if pixels_end > data.len() {
        return None;
    }

    let mut pixels = Vec::with_capacity(count * 4);
    for index in 0..count {
        let argb = read_u32(data, pixels_start + index * 4)?;
        let a = (argb >> 24) as u8;
        let r = ((argb >> 16) & 0xff) as u8;
        let g = ((argb >> 8) & 0xff) as u8;
        let b = (argb & 0xff) as u8;

        // Xcursor stores straight alpha; Smithay blends with premultiplied alpha.
        // Its Argb8888 buffers expect B, G, R, A byte order in memory.
        pixels.extend_from_slice(&[premultiply(b, a), premultiply(g, a), premultiply(r, a), a]);
    }

    Some(CursorImage {
        width,
        height,
        xhot,
        yhot,
        pixels,
    })
}

#[cfg(test)]
mod tests;
