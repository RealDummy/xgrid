use crate::grid::{GridBuilder, GridHandle};
use crate::manager::{BBox, MarginBox};
use crate::{FrameHandle};

pub struct FrameMessage {
    pub size: Option<BBox>,
    pub color: Option<[u8; 4]>,
    pub margin: Option<MarginBox>,
}
pub struct GridMessage {}

pub enum UpdateMessage {
    ModifyFrame(FrameHandle, FrameMessage),
    NewFrame(GridHandle, FrameMessage),
    ModifyGrid(GridHandle, GridMessage),
    NewGrid(GridBuilder),
    Prepare,
    Draw,
}
