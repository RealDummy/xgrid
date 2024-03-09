use std::{process::exit, sync::mpsc};

use log::warn;

use crate::{render_actor::UpdateMessage, UpdateMsg};

use super::back::{SystemUpdates, Update};

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
    pub fn send(&self, msg: Update) {
        match msg {
            Update::User(msg, dst ) => match msg {
                UpdateMsg::Frame(f) => {
                    if let Err(e) = self.sender.send(UpdateMessage::ModifyFrame(dst.frame(), f)) {
                        warn!("{e}");
                        exit(0);
                    }
                }
                _ => (),
            }
            Update::System(msg) => match  msg {
                SystemUpdates::Resized(logical_size, scale_factor) => {
                    self.sender.send(UpdateMessage::ResizeWindow(logical_size, scale_factor)).unwrap();
                }
            }
        }
        
    }
}
