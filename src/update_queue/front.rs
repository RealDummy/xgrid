use std::{process::exit, sync::mpsc};

use log::warn;

use crate::render_actor::UpdateMessage;

use super::back::{QualifiedUpdateMsg, UpdateMsg};

#[derive(Clone)]
pub struct UpdateQueue {
    sender: mpsc::Sender<UpdateMessage>,
}

impl UpdateQueue {
    pub fn new(sender: &mpsc::Sender<UpdateMessage>) -> Self {
        Self {
            sender: sender.clone(),
        }
    }
    pub fn send(&self, msg: QualifiedUpdateMsg) {
        let QualifiedUpdateMsg { msg, dst } = msg;
        use UpdateMsg::*;
        match msg {
            Frame(f) => {
                if let Err(e) = self.sender.send(UpdateMessage::ModifyFrame(dst.frame(), f)) {
                    warn!("{e}");
                    exit(0);
                }
            }
            _ => (),
        }
    }
}
