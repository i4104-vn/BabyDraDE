use std::path::Path;

/// Filesystem usage snapshot for a mount point.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub used_percent: f32,
}

/// Byte-to-gigabyte convenience conversions for disk readings.
impl DiskInfo {
    /// Returns total space in gibibytes.
    ///
    /// # Returns
    /// `total_bytes` converted to GiB.
    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns used space in gibibytes.
    ///
    /// # Returns
    /// `used_bytes` converted to GiB.
    pub fn used_gb(&self) -> f64 {
        self.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns available space in gibibytes.
    ///
    /// # Returns
    /// `available_bytes` converted to GiB.
    pub fn available_gb(&self) -> f64 {
        self.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

/// Queries filesystem statistics for a configured mount point.
#[derive(Debug, Clone)]
pub struct DiskReader {
    mount_point: String,
}

impl Default for DiskReader {
    fn default() -> Self {
        Self::new("/")
    }
}

impl DiskReader {
    /// Creates a reader for the given mount point.
    ///
    /// # Arguments
    /// * `mount_point` - Filesystem path to query.
    pub fn new(mount_point: &str) -> Self {
        Self {
            mount_point: mount_point.to_string(),
        }
    }

    /// Reads statistics for the configured mount point.
    ///
    /// # Returns
    /// Usage information, or `None` if the path is unavailable.
    pub fn read(&self) -> Option<DiskInfo> {
        Self::read_path(Path::new(&self.mount_point))
    }

    #[cfg(unix)]
    #[allow(clippy::unnecessary_cast)]
    /// Queries filesystem statistics for an arbitrary path via `statvfs`.
    ///
    /// # Arguments
    /// * `path` - Path on the filesystem to query.
    ///
    /// # Returns
    /// Usage information, or `None` if the query fails.
    pub fn read_path(path: &Path) -> Option<DiskInfo> {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        use std::os::unix::ffi::OsStrExt;

        let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();

        let ret = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
        if ret != 0 {
            return None;
        }

        let stat = unsafe { stat.assume_init() };
        let block_size = stat.f_frsize as u64;
        let total_bytes = (stat.f_blocks as u64) * block_size;
        let free_bytes = (stat.f_bfree as u64) * block_size;
        let available_bytes = (stat.f_bavail as u64) * block_size;
        let used_bytes = total_bytes.saturating_sub(free_bytes);

        let used_percent = match total_bytes > 0 {
            true => (used_bytes as f64 / total_bytes as f64 * 100.0) as f32,
            false => 0.0,
        };

        Some(DiskInfo {
            mount_point: path.to_string_lossy().to_string(),
            total_bytes,
            available_bytes,
            used_bytes,
            free_bytes,
            used_percent: used_percent.clamp(0.0, 100.0),
        })
    }

    /// Queries filesystem statistics for an arbitrary path (non-Unix stub).
    ///
    /// # Arguments
    /// * `path` - Path on the filesystem to query.
    ///
    /// # Returns
    /// Placeholder usage information.
    #[cfg(not(unix))]
    pub fn read_path(path: &Path) -> Option<DiskInfo> {
        Some(DiskInfo {
            mount_point: path.to_string_lossy().to_string(),
            total_bytes: 100 * 1024 * 1024 * 1024,
            available_bytes: 50 * 1024 * 1024 * 1024,
            used_bytes: 50 * 1024 * 1024 * 1024,
            free_bytes: 50 * 1024 * 1024 * 1024,
            used_percent: 50.0,
        })
    }
}
