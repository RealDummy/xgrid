use std::{borrow::BorrowMut, clone, fmt::Debug, rc::Rc, sync::Mutex};

use log::{debug, warn};

use crate::{frame::{FrameData, FrameHandle}, handle::HandleLike, manager::UpdateManager};


pub enum UpdateAction {
    Rebuild,
    None,
}
pub trait Update {
    type Msg: Debug;
    fn update(&mut self, msg: Self::Msg, frame: FrameHandle, manager: &mut UpdateManager) -> bool;
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager);
    fn init(frame: FrameHandle, manager: &mut UpdateManager) -> Self;
}

pub trait UpdateComponent {
    type Msg: Debug;
    fn update(&self, msg: Self::Msg, frame: FrameHandle, manager: &mut UpdateManager);
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager);
}

#[derive(Debug)]
pub enum Interaction {
    Click(bool),
    Hover,
}

pub trait Frame {
    
}


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
    component: Rc<Mutex<Component<S>>>,
}
impl<S: Update + 'static> ComponentHandle<S> {
    pub(super) fn new(frame: FrameHandle, state: S) -> Self {
        Self {
            component: Rc::new(Mutex::new(Component{
                state: state,
                dirty: true,
                ids: QueryId {
                    handle: frame,
                    class: 0,
                }
            })),
        }
    }
    pub(super) fn as_frame(&self) -> Rc<Mutex<dyn Frame>> {
        self.component.clone()
    }
}

impl<S: Update> UpdateComponent for ComponentHandle<S> {
    type Msg = S::Msg;
    fn update(&self, msg: Self::Msg, frame: FrameHandle, manager: &mut UpdateManager) {
        loop {
            match self.component.lock() {
                Ok(mut state) => {
                    let handle = state.ids.handle;
                    if state.state.update(msg, handle, manager) {
                        state.state.build(handle, manager)
                    }
                    return;
                }
                Err(e) => {
                    warn!("{e}");
                }
            }
        }
    }
    fn build(&self,frame: FrameHandle, manager: &mut UpdateManager) {
        loop {
            match self.component.lock() {
                Ok(state) => {
                    state.state.build(frame, manager);
                    return;
                }
                Err(e) => {
                    warn!("{e}");
                }
            }

        }
    }
}
