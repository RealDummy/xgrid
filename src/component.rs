use std::{borrow::BorrowMut, clone, fmt::Debug, rc::Rc, sync::Mutex};

use log::warn;

use crate::{frame::FrameHandle, manager::{self, UpdateManager}};

pub trait Update {
    type Msg: Debug;
    fn update(&mut self, msg: Self::Msg);
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager);

}

trait UpdateComponent {
    type Msg: Debug;
    fn update(&self, msg: Self::Msg);
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager);
}

#[derive(Debug)]
pub enum Interaction {
    Click(bool),
    Hover,
}

pub trait Frame {
    
}

impl<S: Update> UpdateComponent for ComponentHandle<S> {
    type Msg = S::Msg;
    fn update(&self, msg: Self::Msg) {
        loop {
            match self.component.lock() {
                Ok(mut state) => {
                    state.state.update(msg);
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

pub struct QueryId {
    handle: FrameHandle,
    class: u32,
}

pub(crate) struct Component<S: Update> {
    pub state: S,
    pub dirty: bool,
}

impl<S: Update> Frame for Component<S> {}


pub struct ComponentHandle<S: Update> {
    pub(crate) index: usize,
    pub(crate) component: Rc<Mutex<Component<S>>>,
}
impl<S: Update> ComponentHandle<S> {
    pub(crate) fn new(index: usize, state: S) -> Self {
        Self {
            index,
            component: Rc::new(Mutex::new(Component{
                state: state,
                dirty: true,
            })),
        }
    }
}
