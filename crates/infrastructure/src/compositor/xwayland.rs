/// XWayland integration skeleton for running legacy X11 applications.
#[derive(Debug, Default)]
pub struct XWaylandBridge {
    pub enabled: bool,
    pub display_number: Option<u32>,
}

impl XWaylandBridge {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            display_number: None,
        }
    }

    pub fn start(&mut self) -> Result<u32, String> {
        if !self.enabled {
            return Err("XWayland is disabled in config".into());
        }
        let disp = 0;
        self.display_number = Some(disp);
        tracing::info!("XWayland bridge initialized on DISPLAY=:{disp}");
        Ok(disp)
    }

    pub fn stop(&mut self) {
        if let Some(disp) = self.display_number.take() {
            tracing::info!("XWayland bridge stopped on DISPLAY=:{disp}");
        }
    }
}
