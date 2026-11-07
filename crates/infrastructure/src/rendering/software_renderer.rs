use domain::{CanvasRenderer, Color, Point, Rect, RenderError};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};

#[repr(C)]
#[derive(Default, Debug)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default, Debug)]
struct FbVarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Default, Debug)]
struct FbFixScreeninfo {
    id: [u8; 16],
    smem_start: usize,
    smem_len: u32,
    type_: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    pad: [u8; 2],
    line_length: u32,
    mmio_start: usize,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

#[cfg(unix)]
extern "C" {
    fn ioctl(fd: std::os::raw::c_int, request: std::os::raw::c_ulong, ...) -> std::os::raw::c_int;
    fn mmap(
        addr: *mut std::ffi::c_void,
        len: usize,
        prot: std::os::raw::c_int,
        flags: std::os::raw::c_int,
        fd: std::os::raw::c_int,
        offset: i64,
    ) -> *mut std::ffi::c_void;
    fn munmap(addr: *mut std::ffi::c_void, len: usize) -> std::os::raw::c_int;
}

/// Hardware / Software 2D Canvas Surface Renderer for BabyDraDE.
/// Implements `CanvasRenderer` using mmap direct memory access and exact
/// hardware stride alignment to eliminate tearing and flickering.
pub struct SoftwareRenderer {
    pub width: u32,
    pub height: u32,
    pub line_length: u32,
    pub bits_per_pixel: u32,
    pub buffer: Vec<u32>,
    pub staging_buffer: Vec<u8>,
    pub fb_file: Option<File>,
    pub mmap_addr: usize,
    pub mmap_len: usize,
    pub frame_count: u64,
}

impl SoftwareRenderer {
    /// Creates a new Canvas pixel surface with specified dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            line_length: width * 4,
            bits_per_pixel: 32,
            buffer: vec![0xFF1E1E2E; size],
            staging_buffer: Vec::new(),
            fb_file: None,
            mmap_addr: 0,
            mmap_len: 0,
            frame_count: 0,
        }
    }

    /// Automatically detects physical screen resolution and connects to /dev/fb0 on Linux TTY,
    /// setting up mmap zero-copy memory access to prevent display flickering.
    pub fn auto_detect() -> Self {
        let mut width = 1280;
        let mut height = 720;
        let mut line_length = 0;
        let mut bits_per_pixel = 32;
        let mut smem_len = 0usize;

        let fb_file = OpenOptions::new().read(true).write(true).open("/dev/fb0").ok();

        // 1. Query exact hardware dimensions and stride via Linux ioctls
        #[cfg(unix)]
        if let Some(file) = &fb_file {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();

            // FBIOGET_VSCREENINFO = 0x4600
            let mut vinfo = FbVarScreeninfo::default();
            if unsafe { ioctl(fd, 0x4600, &mut vinfo) } == 0 {
                if vinfo.xres > 0 && vinfo.yres > 0 {
                    width = vinfo.xres;
                    height = vinfo.yres;
                    bits_per_pixel = if vinfo.bits_per_pixel > 0 { vinfo.bits_per_pixel } else { 32 };
                    tracing::info!(
                        "ioctl(FBIOGET_VSCREENINFO): {}x{}, {} bpp (virtual: {}x{})",
                        width,
                        height,
                        bits_per_pixel,
                        vinfo.xres_virtual,
                        vinfo.yres_virtual
                    );
                }
            }

            // FBIOGET_FSCREENINFO = 0x4602
            let mut finfo = FbFixScreeninfo::default();
            if unsafe { ioctl(fd, 0x4602, &mut finfo) } == 0 {
                if finfo.line_length > 0 {
                    line_length = finfo.line_length;
                    smem_len = finfo.smem_len as usize;
                    tracing::info!(
                        "ioctl(FBIOGET_FSCREENINFO): stride={} bytes, smem_len={}",
                        line_length,
                        smem_len
                    );
                }
            }
        }

        // 2. Sysfs fallback if ioctl didn't get all fields
        if line_length == 0 {
            if let Ok(content) = std::fs::read_to_string("/sys/class/graphics/fb0/stride") {
                if let Ok(stride) = content.trim().parse::<u32>() {
                    if stride > 0 {
                        line_length = stride;
                    }
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/sys/class/graphics/fb0/virtual_size") {
            let parts: Vec<&str> = content.trim().split(',').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    if w > 0 && h > 0 {
                        if width == 1280 && height == 720 {
                            width = w;
                            height = h;
                        }
                    }
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/sys/class/graphics/fb0/bits_per_pixel") {
            if let Ok(bpp) = content.trim().parse::<u32>() {
                if bpp > 0 {
                    bits_per_pixel = bpp;
                }
            }
        }

        if line_length == 0 {
            let bytes_per_px = (bits_per_pixel / 8).max(1);
            line_length = width * bytes_per_px;
        }

        let needed_len = (height as usize) * (line_length as usize);
        if smem_len < needed_len {
            smem_len = needed_len;
        }

        // 3. Attempt mmap for direct zero-latency VRAM access (eliminates flickering)
        let mut mmap_addr = 0usize;
        let mut mmap_len = 0usize;

        #[cfg(unix)]
        if let Some(file) = &fb_file {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            // PROT_READ (1) | PROT_WRITE (2), MAP_SHARED (1)
            let ptr = unsafe {
                mmap(
                    std::ptr::null_mut(),
                    smem_len,
                    1 | 2,
                    1,
                    fd,
                    0,
                )
            };
            // Check if mmap failed (MAP_FAILED = -1)
            if ptr as isize != -1 && !ptr.is_null() {
                mmap_addr = ptr as usize;
                mmap_len = smem_len;
                tracing::info!(
                    "Direct mmap framebuffer enabled (address: 0x{:x}, len: {})",
                    mmap_addr,
                    mmap_len
                );
            }
        }

        tracing::info!(
            "Hardware Canvas Display: {}x{}, {} bpp, stride: {} bytes (mmap: {})",
            width,
            height,
            bits_per_pixel,
            line_length,
            mmap_addr != 0
        );

        let mut renderer = Self::new(width, height);
        renderer.line_length = line_length;
        renderer.bits_per_pixel = bits_per_pixel;
        renderer.fb_file = fb_file;
        renderer.mmap_addr = mmap_addr;
        renderer.mmap_len = mmap_len;
        renderer
    }

    /// Helper to convert domain::Color (RGBA) into packed ARGB 32-bit pixel: 0xAARRGGBB
    #[inline]
    pub fn pack_color(color: Color) -> u32 {
        ((color.a as u32) << 24)
            | ((color.r as u32) << 16)
            | ((color.g as u32) << 8)
            | (color.b as u32)
    }

    /// Alpha blends a source color onto an existing destination ARGB pixel on canvas.
    #[inline]
    fn blend_pixel(src: Color, dst: u32) -> u32 {
        if src.a == 255 {
            return Self::pack_color(src);
        }
        if src.a == 0 {
            return dst;
        }

        let sa = src.a as u32;
        let da = 255 - sa;

        let sr = src.r as u32;
        let sg = src.g as u32;
        let sb = src.b as u32;

        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let r = (sr * sa + dr * da) / 255;
        let g = (sg * sa + dg * da) / 255;
        let b = (sb * sa + db * da) / 255;

        (0xFF << 24) | (r << 16) | (g << 8) | b
    }

    /// Exports current canvas state to a PPM image file for testing/verification.
    pub fn save_ppm(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "P6\n{} {}\n255", self.width, self.height)?;
        let mut bytes = Vec::with_capacity((self.width * self.height * 3) as usize);
        for &px in &self.buffer {
            bytes.push(((px >> 16) & 0xFF) as u8);
            bytes.push(((px >> 8) & 0xFF) as u8);
            bytes.push((px & 0xFF) as u8);
        }
        file.write_all(&bytes)?;
        Ok(())
    }
}

impl Drop for SoftwareRenderer {
    fn drop(&mut self) {
        #[cfg(unix)]
        if self.mmap_addr != 0 && self.mmap_len != 0 {
            unsafe {
                munmap(self.mmap_addr as *mut std::ffi::c_void, self.mmap_len);
            }
            self.mmap_addr = 0;
            self.mmap_len = 0;
        }
    }
}

impl CanvasRenderer for SoftwareRenderer {
    fn clear(&mut self, color: Color) -> Result<(), RenderError> {
        let px = Self::pack_color(color);
        self.buffer.fill(px);
        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), RenderError> {
        if color.a == 0 || rect.width() == 0 || rect.height() == 0 {
            return Ok(());
        }

        let x0 = rect.x().max(0) as u32;
        let y0 = rect.y().max(0) as u32;
        let x1 = ((rect.x() + rect.width() as i32).max(0) as u32).min(self.width);
        let y1 = ((rect.y() + rect.height() as i32).max(0) as u32).min(self.height);

        if x0 >= x1 || y0 >= y1 {
            return Ok(());
        }

        if color.a == 255 {
            let px = Self::pack_color(color);
            for y in y0..y1 {
                let row_start = (y * self.width + x0) as usize;
                let row_end = (y * self.width + x1) as usize;
                self.buffer[row_start..row_end].fill(px);
            }
        } else {
            for y in y0..y1 {
                let row_offset = (y * self.width) as usize;
                for x in x0..x1 {
                    let idx = row_offset + x as usize;
                    self.buffer[idx] = Self::blend_pixel(color, self.buffer[idx]);
                }
            }
        }

        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        _font_size: f32,
        color: Color,
    ) -> Result<(), RenderError> {
        if color.a == 0 || text.is_empty() {
            return Ok(());
        }

        let mut cur_x = position.x;
        let mut cur_y = position.y;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = position.x;
                cur_y += 18;
                continue;
            }

            draw_bitmap_glyph(self, ch, cur_x, cur_y, color);
            cur_x += 9;
        }

        Ok(())
    }

    fn render_frame(&mut self) -> Result<(), RenderError> {
        self.frame_count += 1;

        // 1. Direct mmap fast blit (smooth, zero-syscall, no flicker)
        if self.mmap_addr != 0 {
            let mmap_ptr = self.mmap_addr as *mut u8;

            if self.bits_per_pixel == 32 {
                if self.line_length == self.width * 4 {
                    // Contiguous single-shot memcpy
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            self.buffer.as_ptr() as *const u8,
                            mmap_ptr,
                            (self.width * self.height * 4) as usize,
                        );
                    }
                } else {
                    // Row-by-row stride memcpy
                    for y in 0..self.height {
                        let dst_offset = (y as usize) * (self.line_length as usize);
                        let src_start = (y * self.width) as usize;
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                self.buffer[src_start..].as_ptr() as *const u8,
                                mmap_ptr.add(dst_offset),
                                (self.width * 4) as usize,
                            );
                        }
                    }
                }
            } else if self.bits_per_pixel == 16 {
                // 16-bit RGB565 conversion
                let mut row16 = vec![0u16; self.width as usize];
                for y in 0..self.height {
                    let dst_offset = (y as usize) * (self.line_length as usize);
                    let src_start = (y * self.width) as usize;
                    for x in 0..self.width as usize {
                        let px32 = self.buffer[src_start + x];
                        let r = ((px32 >> 16) & 0xFF) as u16;
                        let g = ((px32 >> 8) & 0xFF) as u16;
                        let b = (px32 & 0xFF) as u16;
                        let r5 = (r >> 3) & 0x1F;
                        let g6 = (g >> 2) & 0x3F;
                        let b5 = (b >> 3) & 0x1F;
                        row16[x] = (r5 << 11) | (g6 << 5) | b5;
                    }
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            row16.as_ptr() as *const u8,
                            mmap_ptr.add(dst_offset),
                            (self.width * 2) as usize,
                        );
                    }
                }
            }
            return Ok(());
        }

        // 2. Fallback to pwrite / write_all if mmap is unavailable
        if let Some(fb) = &mut self.fb_file {
            if self.bits_per_pixel == 32 {
                if self.line_length == self.width * 4 {
                    let _ = fb.seek(SeekFrom::Start(0));
                    let all_bytes = unsafe {
                        std::slice::from_raw_parts(
                            self.buffer.as_ptr() as *const u8,
                            self.buffer.len() * 4,
                        )
                    };
                    let _ = fb.write_all(all_bytes);
                } else {
                    let total_bytes = (self.height as usize) * (self.line_length as usize);
                    if self.staging_buffer.len() != total_bytes {
                        self.staging_buffer.resize(total_bytes, 0);
                    }
                    let row_bytes = (self.width as usize) * 4;
                    for y in 0..self.height as usize {
                        let dst_offset = y * (self.line_length as usize);
                        let src_start = y * (self.width as usize);
                        let src_slice: &[u8] = unsafe {
                            std::slice::from_raw_parts(
                                self.buffer[src_start..].as_ptr() as *const u8,
                                row_bytes,
                            )
                        };
                        self.staging_buffer[dst_offset..dst_offset + row_bytes]
                            .copy_from_slice(src_slice);
                    }
                    let _ = fb.seek(SeekFrom::Start(0));
                    let _ = fb.write_all(&self.staging_buffer);
                }
            } else if self.bits_per_pixel == 16 {
                let total_bytes = (self.height as usize) * (self.line_length as usize);
                if self.staging_buffer.len() != total_bytes {
                    self.staging_buffer.resize(total_bytes, 0);
                }
                for y in 0..self.height as usize {
                    let dst_offset = y * (self.line_length as usize);
                    let src_start = y * (self.width as usize);
                    for x in 0..self.width as usize {
                        let px32 = self.buffer[src_start + x];
                        let r = ((px32 >> 16) & 0xFF) as u16;
                        let g = ((px32 >> 8) & 0xFF) as u16;
                        let b = (px32 & 0xFF) as u16;
                        let r5 = (r >> 3) & 0x1F;
                        let g6 = (g >> 2) & 0x3F;
                        let b5 = (b >> 3) & 0x1F;
                        let rgb565 = (r5 << 11) | (g6 << 5) | b5;
                        let bytes = rgb565.to_ne_bytes();
                        self.staging_buffer[dst_offset + x * 2] = bytes[0];
                        self.staging_buffer[dst_offset + x * 2 + 1] = bytes[1];
                    }
                }
                let _ = fb.seek(SeekFrom::Start(0));
                let _ = fb.write_all(&self.staging_buffer);
            }
            let _ = fb.flush();
        }

        Ok(())
    }
}

/// 8x16 bitmap font glyph rendering onto the canvas surface
fn draw_bitmap_glyph(renderer: &mut SoftwareRenderer, ch: char, x: i32, y: i32, color: Color) {
    let glyph = get_glyph_data(ch);
    let px = SoftwareRenderer::pack_color(color);

    for (row_idx, &row_byte) in glyph.iter().enumerate() {
        let py = y + row_idx as i32;
        if py < 0 || py >= renderer.height as i32 {
            continue;
        }

        for col_idx in 0..8 {
            if (row_byte & (0x80 >> col_idx)) != 0 {
                let px_x = x + col_idx;
                if px_x >= 0 && px_x < renderer.width as i32 {
                    let idx = (py as u32 * renderer.width + px_x as u32) as usize;
                    if idx < renderer.buffer.len() {
                        renderer.buffer[idx] = px;
                    }
                }
            }
        }
    }
}

/// Embedded glyph patterns (8x16) for printable ASCII characters
fn get_glyph_data(ch: char) -> [u8; 16] {
    match ch {
        'A' => [0, 0, 0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0],
        'B' => [0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0, 0, 0, 0, 0, 0, 0],
        'C' => [0, 0, 0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'D' => [0, 0, 0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0, 0, 0, 0, 0, 0, 0],
        'E' => [0, 0, 0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0, 0, 0, 0, 0, 0, 0],
        'F' => [0, 0, 0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0, 0, 0, 0, 0, 0, 0],
        'G' => [0, 0, 0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'H' => [0, 0, 0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'I' => [0, 0, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'J' => [0, 0, 0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0, 0, 0, 0, 0, 0, 0],
        'K' => [0, 0, 0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'L' => [0, 0, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0, 0, 0, 0, 0, 0, 0],
        'M' => [0, 0, 0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0, 0, 0, 0, 0, 0, 0],
        'N' => [0, 0, 0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'O' => [0, 0, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'P' => [0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0, 0, 0, 0, 0, 0, 0],
        'Q' => [0, 0, 0x3C, 0x66, 0x66, 0x66, 0x6A, 0x64, 0x3A, 0, 0, 0, 0, 0, 0, 0],
        'R' => [0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x78, 0x6C, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'S' => [0, 0, 0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'T' => [0, 0, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0],
        'U' => [0, 0, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'V' => [0, 0, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0, 0, 0, 0, 0, 0, 0],
        'W' => [0, 0, 0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0, 0, 0, 0, 0, 0, 0],
        'X' => [0, 0, 0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'Y' => [0, 0, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0],
        'Z' => [0, 0, 0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0, 0, 0, 0, 0, 0, 0],

        'a' => [0, 0, 0, 0, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0, 0, 0, 0, 0, 0, 0],
        'b' => [0, 0, 0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0, 0, 0, 0, 0, 0, 0],
        'c' => [0, 0, 0, 0, 0x3C, 0x66, 0x60, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'd' => [0, 0, 0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0, 0, 0, 0, 0, 0, 0],
        'e' => [0, 0, 0, 0, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'f' => [0, 0, 0x1C, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x30, 0, 0, 0, 0, 0, 0, 0],
        'g' => [0, 0, 0, 0, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x3C, 0, 0, 0, 0, 0, 0],
        'h' => [0, 0, 0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'i' => [0, 0, 0x18, 0, 0x38, 0x18, 0x18, 0x18, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'j' => [0, 0, 0x0C, 0, 0x1C, 0x0C, 0x0C, 0x0C, 0x4C, 0x38, 0, 0, 0, 0, 0, 0],
        'k' => [0, 0, 0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'l' => [0, 0, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'm' => [0, 0, 0, 0, 0x66, 0x7F, 0x6B, 0x63, 0x63, 0, 0, 0, 0, 0, 0, 0],
        'n' => [0, 0, 0, 0, 0x7C, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'o' => [0, 0, 0, 0, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        'p' => [0, 0, 0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0, 0, 0, 0, 0, 0],
        'q' => [0, 0, 0, 0, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06, 0, 0, 0, 0, 0, 0],
        'r' => [0, 0, 0, 0, 0x7C, 0x66, 0x60, 0x60, 0x60, 0, 0, 0, 0, 0, 0, 0],
        's' => [0, 0, 0, 0, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0, 0, 0, 0, 0, 0, 0],
        't' => [0, 0, 0x30, 0x30, 0x7C, 0x30, 0x30, 0x32, 0x1C, 0, 0, 0, 0, 0, 0],
        'u' => [0, 0, 0, 0, 0x66, 0x66, 0x66, 0x66, 0x3E, 0, 0, 0, 0, 0, 0, 0],
        'v' => [0, 0, 0, 0, 0x66, 0x66, 0x66, 0x3C, 0x18, 0, 0, 0, 0, 0, 0, 0],
        'w' => [0, 0, 0, 0, 0x63, 0x6B, 0x7F, 0x3E, 0x36, 0, 0, 0, 0, 0, 0, 0],
        'x' => [0, 0, 0, 0, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0, 0, 0, 0, 0, 0, 0],
        'y' => [0, 0, 0, 0, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x3C, 0, 0, 0, 0, 0, 0],
        'z' => [0, 0, 0, 0, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0, 0, 0, 0, 0, 0, 0],

        '0' => [0, 0, 0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        '1' => [0, 0, 0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0, 0, 0, 0, 0, 0, 0],
        '2' => [0, 0, 0x3C, 0x66, 0x06, 0x1C, 0x30, 0x60, 0x7E, 0, 0, 0, 0, 0, 0, 0],
        '3' => [0, 0, 0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        '4' => [0, 0, 0x0C, 0x1C, 0x34, 0x64, 0x7E, 0x04, 0x0E, 0, 0, 0, 0, 0, 0, 0],
        '5' => [0, 0, 0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        '6' => [0, 0, 0x3C, 0x66, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        '7' => [0, 0, 0x7E, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0, 0, 0, 0, 0, 0, 0],
        '8' => [0, 0, 0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        '9' => [0, 0, 0x3C, 0x66, 0x66, 0x3E, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0, 0, 0],

        '[' => [0, 0, 0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        ']' => [0, 0, 0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0, 0, 0, 0, 0, 0, 0],
        '(' => [0, 0, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0, 0, 0, 0, 0, 0, 0],
        ')' => [0, 0, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0, 0, 0, 0, 0, 0, 0],
        '{' => [0, 0, 0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0, 0, 0, 0, 0, 0, 0],
        '}' => [0, 0, 0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0, 0, 0, 0, 0, 0, 0],
        '<' => [0, 0, 0x06, 0x1C, 0x70, 0x1C, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '>' => [0, 0, 0x60, 0x38, 0x0E, 0x38, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7E, 0, 0, 0, 0, 0, 0],
        '-' => [0, 0, 0, 0, 0x7E, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '+' => [0, 0, 0x18, 0x18, 0x7E, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '=' => [0, 0, 0, 0x7E, 0, 0x7E, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ':' => [0, 0, 0, 0x30, 0x30, 0, 0x30, 0x30, 0, 0, 0, 0, 0, 0, 0, 0],
        ';' => [0, 0, 0, 0x30, 0x30, 0, 0x30, 0x10, 0x20, 0, 0, 0, 0, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 0, 0x30, 0x30, 0, 0, 0, 0, 0, 0, 0, 0],
        ',' => [0, 0, 0, 0, 0, 0, 0x30, 0x10, 0x20, 0, 0, 0, 0, 0, 0, 0],
        '/' => [0, 0, 0x06, 0x0C, 0x18, 0x30, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '\\' => [0, 0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '|' => [0, 0, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0],
        '*' => [0, 0, 0x24, 0x18, 0x7E, 0x18, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '!' => [0, 0, 0x18, 0x18, 0x18, 0x18, 0, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0],
        '?' => [0, 0, 0x3C, 0x66, 0x06, 0x1C, 0x18, 0, 0x18, 0, 0, 0, 0, 0, 0, 0],
        '@' => [0, 0, 0x3C, 0x66, 0x6E, 0x6B, 0x6E, 0x60, 0x3E, 0, 0, 0, 0, 0, 0, 0],
        '#' => [0, 0, 0x24, 0x24, 0x7E, 0x24, 0x7E, 0x24, 0x24, 0, 0, 0, 0, 0, 0, 0],
        '$' => [0, 0x18, 0x3E, 0x58, 0x3C, 0x1A, 0x7C, 0x18, 0, 0, 0, 0, 0, 0, 0, 0],
        '%' => [0, 0, 0x62, 0x64, 0x08, 0x10, 0x26, 0x46, 0, 0, 0, 0, 0, 0, 0, 0],
        '&' => [0, 0, 0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0, 0, 0, 0, 0, 0, 0],
        '\'' => [0, 0x18, 0x18, 0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '"' => [0, 0x36, 0x36, 0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '~' => [0, 0, 0x36, 0x5C, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        '^' => [0, 0x18, 0x3C, 0x66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ' ' => [0; 16],
        _ => [0, 0, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0, 0, 0, 0, 0, 0, 0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_renderer_pipeline() {
        let mut r = SoftwareRenderer::new(800, 600);
        assert_eq!(r.width, 800);
        assert_eq!(r.height, 600);
        assert_eq!(r.line_length, 800 * 4);

        // Clear canvas
        assert!(r.clear(Color::rgb(20, 20, 30)).is_ok());

        // Draw rect on canvas
        let rect = Rect::new(50, 50, 200, 100);
        assert!(r.draw_rect(rect, Color::rgb(255, 0, 0)).is_ok());

        // Draw text on canvas
        assert!(r
            .draw_text("BabyDraDE Canvas", Point::new(60, 60), 14.0, Color::WHITE)
            .is_ok());

        // Render frame to canvas
        assert!(r.render_frame().is_ok());
        assert_eq!(r.frame_count, 1);
    }
}
