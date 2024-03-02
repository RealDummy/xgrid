use std::{borrow::BorrowMut, clone, fmt::Debug, marker::PhantomData, rc::Rc, sync::Mutex};

use log::{debug, warn};

use crate::{
    frame::{FrameData, FrameHandle},
    handle::HandleLike,
    manager::UpdateManager,
    render_actor::UpdateMessage,
};

pub enum UpdateAction {
    Rebuild,
    None,
}
pub trait Update {
    type Msg: Debug;
    fn update(&mut self, msg: Self::Msg) -> bool;
    fn build(&self);
    fn init() -> Self;
}

#[derive(Debug)]
pub enum Interaction {
    Click(bool),
    Hover,
}

pub trait Frame {}

pub struct QueryId {
    handle: FrameHandle,
    class: u32,
}

pub struct Component<S: Update> {
    state: S,
    ids: QueryId,
    dirty: bool,
}

impl<S: Update> Frame for Component<S> {}

pub struct ComponentHandle<S: Update> {
    state_index: usize,
    frame_index: FrameHandle,
    _t: PhantomData<S>,
}
impl<S: Update> ComponentHandle<S> {
    pub(super) fn new(state: S) -> Self {
        todo!()
    }
}
