use std::{fmt::Debug, marker::PhantomData, sync::mpsc};



use log::debug;

use crate::{
    frame::FrameHandle, grid::{GridBuilder, GridHandle, XName, YName}, handle::HandleLike, manager::{Borders, MarginBox, Rect}, render_actor::{FrameMessage, UpdateMessage}, units::VUnit, update_queue::{self, back::QualifiedUpdateMsg}, UpdateMsg
};

pub struct ComponentBuilder {
    render_sender: mpsc::Sender<UpdateMessage>,
    frame_count: usize,
    grid_count: usize,
}

pub struct Builder<'a> {
    b: &'a mut ComponentBuilder,
    index: ComponentType,
}
impl ComponentBuilder {
    pub fn new(send: mpsc::Sender<UpdateMessage>) -> Self {
        ComponentBuilder {
            render_sender: send,
            frame_count: 0,
            grid_count: 0,
        }
    }
}

impl<'a> Builder<'a> {
    pub(crate) fn new(b: &'a mut ComponentBuilder, index: ComponentType ) -> Self {
        Self {
            b,
            index
        }
    }
    pub fn frame<T: State>(&mut self, grid: GridHandle, x: Option<XName>, y: Option<YName>) -> Component<T> {
        let res = FrameHandle::new(self.b.frame_count);

        self.b.render_sender.send(UpdateMessage::NewFrame(grid,x,y, FrameMessage {
            size: None,
            margin: Some(Borders {
                top: 10,
                bottom: 10,
                left: 10,
                right: 10,
            }.into()),
            color: Some([255; 4])
        }, res)).unwrap();
        self.b.frame_count += 1;
        Component {
            value: T::init(self),
            handle: ComponentType::GridMember(res, grid),
        }
    }
    pub fn floating_frame<T: State>(&mut self, size: Rect<i32>) -> Component<T> {
        self.b.render_sender.send(UpdateMessage::NewFloatingFrame(FrameMessage {
            size: Some(size.into()),
            margin: None,
            color: Some([255; 4])
        })).unwrap();
        let res = self.b.frame_count;
        self.b.frame_count += 1;
        Component {
            value: T::init(self),
            handle: ComponentType::Floating(FrameHandle::new(res)),
        }
    }
    pub fn grid_builder(&mut self) -> GridBuilder {
        let parent = match self.index {
            ComponentType::Floating(i) => i,
            ComponentType::GridMember(i, _g) => i
        };
        GridBuilder::new(parent)
    }
    pub fn grid(&mut self, grid: GridBuilder) -> GridHandle {
        self.b.render_sender.send(UpdateMessage::NewGrid(GridHandle::new(self.b.grid_count), grid)).unwrap();
        let res = self.b.grid_count;
        self.b.grid_count += 1;
        GridHandle::new(res)
    }
}

pub struct UpdateQueue<'a> {
    q: &'a update_queue::front::UpdateQueue,
    handle: ComponentType,
}
impl<'a> UpdateQueue<'a> {
    pub fn from_base(q: &'a update_queue::front::UpdateQueue, handle: ComponentType) -> Self {
        Self {
            q,
            handle
        }
    }
    pub fn push(&self, msg: UpdateMsg) {
        self.q.send(QualifiedUpdateMsg {
            msg,
            dst: self.handle.clone(),
        });
    }
}

pub trait State {
    type Msg: Debug;
    fn init(builder: &mut Builder) -> Self;
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue);
}

#[derive(Clone, Debug)]
pub enum ComponentType {
    Floating(FrameHandle),
    GridMember(FrameHandle, GridHandle),
}

impl ComponentType {
    pub fn frame(&self) -> FrameHandle {
        match self {
            Self::Floating(f)=>f.clone(),
            Self::GridMember(f,_ )=>f.clone(),
        }
    }
}

pub struct Component<T: State> {
    value: T,
    handle: ComponentType,
}

impl<T: State> Component<T> {
    pub fn update(&mut self, msg: T::Msg, queue: &UpdateQueue) {
        self.value.update(msg, &UpdateQueue {
            q: queue.q,
            handle: self.handle.clone(),
        });
    }
}

#[derive(Debug)]
pub enum Interaction {
    Click(bool),
    Hover,
}
