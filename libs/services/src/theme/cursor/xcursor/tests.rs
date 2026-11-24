use super::*;

fn argb(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn make_xcursor(sizes: &[u32], width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(HEADER_MAGIC);
    write_u32(&mut bytes, HEADER_LEN as u32);
    write_u32(&mut bytes, 1 << 16); // XCURSOR_FILE_VERSION 1.0
    write_u32(&mut bytes, sizes.len() as u32);

    let mut offsets = Vec::new();
    let mut cursor = HEADER_LEN + sizes.len() * TOC_ENTRY_LEN;
    for _ in sizes {
        offsets.push(cursor);
        cursor += IMAGE_HEADER_LEN + (width * height * 4) as usize;
    }

    for (size, offset) in sizes.iter().zip(offsets.iter()) {
        write_u32(&mut bytes, IMAGE_TYPE);
        write_u32(&mut bytes, *size);
        write_u32(&mut bytes, *offset as u32);
    }

    for (size, offset) in sizes.iter().zip(offsets.iter()) {
        bytes.resize(*offset, 0);
        write_u32(&mut bytes, IMAGE_HEADER_LEN as u32);
        write_u32(&mut bytes, IMAGE_TYPE);
        write_u32(&mut bytes, *size);
        write_u32(&mut bytes, 1); // chunk version
        write_u32(&mut bytes, width);
        write_u32(&mut bytes, height);
        write_u32(&mut bytes, 1); // xhot
        write_u32(&mut bytes, 1); // yhot
        write_u32(&mut bytes, 0); // delay
        for _ in 0..width * height {
            // Encode the nominal size in red so tests can tell chunks apart.
            write_u32(&mut bytes, argb(*size as u8, 20, 30, 255));
        }
    }

    bytes
}

#[test]
fn parses_xcursor_header_and_pixels() {
    let data = make_xcursor(&[24, 32], 2, 2);
    let image = parse_xcursor(&data, 24).expect("valid xcursor should parse");

    assert_eq!(image.width, 2);
    assert_eq!(image.height, 2);
    assert_eq!(image.xhot, 1);
    assert_eq!(image.yhot, 1);
    assert_eq!(image.pixels.len(), 16);
    assert_eq!(image.pixels[..4], [30, 20, 24, 255]);
}

#[test]
fn picks_closest_nominal_size() {
    let data = make_xcursor(&[16, 32, 48], 2, 2);
    let image = parse_xcursor(&data, 30).expect("valid xcursor should parse");
    // The 32px entry is closest to the requested 30px.
    assert_eq!(image.pixels[2], 32);
}

#[test]
fn rejects_invalid_files() {
    assert!(parse_xcursor(b"nope", 24).is_none());
    assert!(parse_xcursor(&[], 24).is_none());
    assert!(parse_xcursor(&make_xcursor(&[24], 2, 2)[..20], 24).is_none());
}
