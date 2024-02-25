#![allow(unused)]
use crate::frame::FrameData;

type FrameExposedType = FrameData;

// maybe?
pub trait FrameMessage {
    fn update_state(&self, updater: dyn FnOnce(&mut FrameExposedType)) -> FrameExposedType;
}