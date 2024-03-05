use std::sync::mpsc;

use crate::{
    component::ComponentType,
    grid::{XName, YName},
    manager::BBox,
    render_actor::FrameMessage,
    units::UserUnits,
};

#[derive(Clone)]
pub enum Bounds {
    Rel(BBox),
    Abs(BBox),
}

pub struct QualifiedUpdateMsg {
    pub msg: UpdateMsg,
    pub dst: ComponentType,
}

#[derive(Clone)]
pub enum UpdateMsg {
    Frame(FrameMessage),
    GridX(XName, UserUnits),
    GridY(YName, UserUnits),
}

pub type UpdateSend = mpsc::SyncSender<QualifiedUpdateMsg>;
pub type UpdateRecv = mpsc::Receiver<QualifiedUpdateMsg>;

pub struct UpdateReciever {
    recv: UpdateRecv,
}
impl UpdateReciever {
    pub fn new(recv: UpdateRecv) -> Self {
        Self { recv }
    }
}
