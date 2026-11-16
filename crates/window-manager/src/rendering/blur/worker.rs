//! Background work for blur configuration. OpenGL resources stay on the
//! compositor render thread; only filesystem/environment loading is delegated.

use std::{
    sync::mpsc::{self, Receiver},
    thread,
};

use super::{config, config::BlurParams};

pub(crate) struct ConfigWorker {
    receiver: Option<Receiver<BlurParams>>,
}

impl ConfigWorker {
    pub(crate) fn start() -> Self {
        let (sender, receiver) = mpsc::channel();
        let _ = thread::Builder::new()
            .name("babydra-blur-config".into())
            .spawn(move || {
                let _ = sender.send(config::load());
            });
        Self {
            receiver: Some(receiver),
        }
    }

    pub(crate) fn try_finish(&mut self) -> Option<BlurParams> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(params) => {
                self.receiver = None;
                Some(params)
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                None
            }
            Err(mpsc::TryRecvError::Empty) => None,
        }
    }
}
