use std::sync::mpsc;

use winit::dpi::LogicalSize;

use crate::{
    component::ComponentType,
    grid::{XName, YName},
    manager::BBox,
    render_actor::{FrameMessage, UpdateMessage},
    units::UserUnits,
};

#[derive(Clone)]
pub enum Bounds {
    Rel(BBox),
    Abs(BBox),
}

pub enum SystemUpdates {
    Resized(LogicalSize<u32>, f64)
}

pub enum Update {
    User(UpdateMsg, ComponentType),
    System(SystemUpdates),
}


#[derive(Clone)]
pub enum UpdateMsg {
    Frame(FrameMessage),
    GridX(XName, UserUnits),
    GridY(YName, UserUnits),
}
