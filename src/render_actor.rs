use std::borrow::Borrow;
use std::mem::MaybeUninit;
use std::sync::mpsc::Sender;
use std::sync::{self, mpsc, Arc, Condvar, Mutex};
use std::thread::{self, scope, JoinHandle, Scope, ScopedJoinHandle};

use log::debug;
use wgpu::{CommandEncoder, RenderPass};
use winit::window::Window;

use crate::frame::FrameRenderer;
use crate::grid::{GridBuilder, GridData, GridHandle, GridRenderer};
use crate::manager::{BBox, MarginBox};
use crate::{FrameHandle, UpdateManager};

pub struct FrameMessage {
    pub size: Option<BBox>,
    pub color: Option<[u8; 4]>,
    pub margin: Option<MarginBox>,
}
pub struct GridMessage {}

pub enum UpdateMessage {
    ModifyFrame(FrameHandle, FrameMessage),
    NewFrame(FrameMessage),
    ModifyGrid(GridHandle, GridMessage),
    NewGrid(GridBuilder),
    Prepare,
    Draw,
}
