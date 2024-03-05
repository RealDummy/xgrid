use std::{process::exit, sync::mpsc};

use log::warn;

use crate::{component::ComponentType, grid::{builder::GridDir, GridHandle}, handle, manager::{BBox, MarginBox, Rect}, render_actor::{FrameMessage, UpdateMessage}, units::UserUnits, FrameHandle};
use crate::grid;

use super::back::{QualifiedUpdateMsg, UpdateMsg, UpdateReciever, UpdateSend};


#[derive(Clone)]
pub struct  UpdateQueue {
    sender: mpsc::Sender<UpdateMessage>,
}

impl UpdateQueue {
    pub fn new(sender: &mpsc::Sender<UpdateMessage>) -> Self {
        Self {
            sender: sender.clone()
        }
    }
    pub fn send(&self, msg: QualifiedUpdateMsg) {
        let QualifiedUpdateMsg {
            msg,dst
        } = msg;
        use UpdateMsg::*;
        match msg {
            Frame(f) => {
                if let Err(e) = self.sender.send(UpdateMessage::ModifyFrame(dst.frame(), f)) {
                    warn!("{e}");
                    exit(0);
                }
            },
            _ => ()
        }
    }
}

