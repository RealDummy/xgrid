use crate::grid::{GridBuilder, GridHandle, XName, YName};
use crate::manager::{BBox, MarginBox};
use crate::{FrameHandle};

#[derive(Clone, Default, Debug)]
pub struct FrameMessage {
    pub size: Option<BBox>,
    pub color: Option<[u8; 4]>,
    pub margin: Option<MarginBox>,
}
#[derive(Clone, Default, Debug)]
pub struct GridMessage {}

#[derive(Debug)]
pub enum UpdateMessage {
    ModifyFrame(FrameHandle, FrameMessage),
    NewFrame(GridHandle, Option<XName>, Option<YName>, FrameMessage, FrameHandle),
    NewFloatingFrame(FrameMessage),
    ModifyGrid(GridHandle, GridMessage),
    NewGrid(GridHandle, GridBuilder),
    Prepare,
    Draw,
}
