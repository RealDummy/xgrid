use std::{
    cell::RefCell,
    fmt::Debug,
    sync::{mpsc, Arc},
};

use crate::{
    events::{KeyboardEvent, MouseEvent},
    frame::FrameHandle,
    grid::{GridBuilder, GridHandle, XName, YName},
    handle::HandleLike,
    manager::{BBox, Borders, Rect},
    render_actor::{FrameMessage, UpdateMessage},
    update_queue::{self, back::QualifiedUpdateMsg, front},
    EventDispatcher, Subscriber, UpdateMsg,
};

pub struct SystemEvents {
    mouse_dispatcher: EventDispatcher<MouseEvent>,
    keyboard_dispatcher: EventDispatcher<KeyboardEvent>,
}
impl SystemEvents {
    pub fn add_mouse_observer<C: State + Subscriber<MouseEvent> + 'static>(
        &mut self,
        component: &Component<C>,
    ) {
        self.mouse_dispatcher.register(component)
    }
    pub fn add_keyboard_observer<C: State + Subscriber<KeyboardEvent> + 'static>(
        &mut self,
        component: &Component<C>,
    ) {
        self.keyboard_dispatcher.register(component)
    }
}
pub struct ComponentBuilder {
    render_sender: mpsc::Sender<UpdateMessage>,
    frame_count: usize,
    grid_count: usize,
    dispatcher: SystemEvents,
    queue: front::UpdateQueue,
}

impl State for () {
    type Msg = ();
    type Param = ();
    fn init<P: State>(_builder: &mut Builder<P>, _param: &Self::Param) -> Self {
        ()
    }
    fn update(&mut self, _msg: Self::Msg, _queue: &UpdateQueue) {}
}
pub struct Builder<'a, C: State> {
    b: &'a mut ComponentBuilder,
    parent: Component<C>,
}
impl ComponentBuilder {
    pub fn new(send: mpsc::Sender<UpdateMessage>, queue: front::UpdateQueue) -> Self {
        ComponentBuilder {
            render_sender: send,
            frame_count: 0,
            grid_count: 0,
            dispatcher: SystemEvents {
                mouse_dispatcher: EventDispatcher::new(&queue),
                keyboard_dispatcher: EventDispatcher::new(&queue),
            },
            queue,
        }
    }
    pub fn send_frame(
        &mut self,
        grid: GridHandle,
        x: Option<XName>,
        y: Option<YName>,
    ) -> FrameHandle {
        let res = FrameHandle::new(self.frame_count);
        self.render_sender
            .send(UpdateMessage::NewFrame(
                grid,
                x,
                y,
                FrameMessage {
                    size: None,
                    margin: Some(
                        Borders {
                            top: 10,
                            bottom: 10,
                            left: 10,
                            right: 10,
                        }
                        .into(),
                    ),
                    color: Some([255; 4]),
                },
                res,
            ))
            .unwrap();
        self.frame_count += 1;
        return res;
    }
    pub fn send_floating(&mut self, size: BBox) -> FrameHandle {
        self.render_sender
            .send(UpdateMessage::NewFloatingFrame(FrameMessage {
                size: Some(size.into()),
                margin: None,
                color: Some([255; 4]),
            }))
            .unwrap();
        let res = FrameHandle::new(self.frame_count);
        self.frame_count += 1;
        return res;
    }
    pub fn send_grid(&mut self, grid: GridBuilder) -> GridHandle {
        self.render_sender
            .send(UpdateMessage::NewGrid(
                GridHandle::new(self.grid_count),
                grid,
            ))
            .unwrap();
        let res = GridHandle::new(self.grid_count);
        self.grid_count += 1;
        return res;
    }
    pub(crate) fn send_app<App: State<Param = ()>>(&mut self, size: BBox) -> Component<App> {
        assert!(self.frame_count == 0);
        let res = self.send_floating(size);
        let mut b = Builder::first(self);
        let app = Component::new(App::init(&mut b, &()), ComponentType::Floating(res));
        App::after_init(&app, &mut self.dispatcher, &());
        return app;
    }
    pub(crate) fn emit_mouse(&self, event: MouseEvent) {
        self.dispatcher.mouse_dispatcher.emit(event)
    }
}
impl<'a> Builder<'a, ()> {
    pub(crate) fn first(b: &'a mut ComponentBuilder) -> Self {
        Self {
            b,
            parent: Component::new((), ComponentType::Floating(FrameHandle::new(0))),
        }
    }
}

impl<'a, C: State> Builder<'a, C> {
    pub(crate) fn new(b: &'a mut ComponentBuilder, component: Component<C>) -> Self {
        Self {
            b,
            parent: component,
        }
    }
    pub fn frame<T: State>(
        &mut self,
        param: T::Param,
        grid: GridHandle,
        x: Option<XName>,
        y: Option<YName>,
    ) -> Component<T> {
        let res = self.b.send_frame(grid, x, y);
        let me = Component::new(
            T::init(
                &mut Builder::new(self.b, Component::clone(&self.parent)),
                &param,
            ),
            ComponentType::GridMember(res, grid),
        );
        T::after_init(&me, &mut self.b.dispatcher, &param);
        return me;
    }
    pub fn floating_frame(&mut self, param: C::Param, size: Rect<i32>) -> Component<C> {
        let res = self.b.send_floating(size.into());
        Component::new(C::init(self, &param), ComponentType::Floating(res))
    }
    pub fn grid_builder(&mut self) -> GridBuilder {
        let parent = match self.parent.handle {
            ComponentType::Floating(i) => i,
            ComponentType::GridMember(i, _g) => i,
        };
        GridBuilder::new(parent)
    }
    pub fn grid(&mut self, grid: GridBuilder) -> GridHandle {
        let res = self.b.send_grid(grid);
        return res;
    }
    pub fn event_dispatcher<Event>(&self) -> EventDispatcher<Event> {
        EventDispatcher::new(&self.b.queue)
    }
}

pub struct UpdateQueue<'a> {
    q: &'a update_queue::front::UpdateQueue,
    handle: ComponentType,
}
impl<'a> UpdateQueue<'a> {
    pub fn from_base(q: &'a update_queue::front::UpdateQueue, handle: ComponentType) -> Self {
        Self { q, handle }
    }
    pub fn push(&self, msg: UpdateMsg) {
        self.q.send(QualifiedUpdateMsg {
            msg,
            dst: self.handle.clone(),
        });
    }
}

pub trait State: Sized {
    type Msg: Debug;
    type Param;
    fn init<P: State>(builder: &mut Builder<P>, param: &Self::Param) -> Self;
    fn after_init(
        _component: &Component<Self>,
        _system_events: &mut SystemEvents,
        _param: &Self::Param,
    ) {
    }
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
            Self::Floating(f) => f.clone(),
            Self::GridMember(f, _) => f.clone(),
        }
    }
}

pub type ComponentInner<T> = Arc<RefCell<T>>;

pub struct Component<T: State> {
    pub(crate) inner: ComponentInner<T>,
    pub(crate) handle: ComponentType,
}

impl<T: State> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl<T: State> Component<T> {
    fn new(t: T, handle: ComponentType) -> Self {
        Self {
            inner: Arc::new(RefCell::new(t)),
            handle,
        }
    }
    pub fn update(&mut self, msg: T::Msg, queue: &UpdateQueue) {
        self.inner.borrow_mut().update(
            msg,
            &UpdateQueue {
                q: queue.q,
                handle: self.handle.clone(),
            },
        );
    }
    pub(crate) fn inner(&self) -> ComponentInner<T> {
        self.inner.clone()
    }
}

#[derive(Debug)]
pub enum Interaction {
    Click(bool),
    Hover,
}
