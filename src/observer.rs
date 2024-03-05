use crate::{component::ComponentInner, Component, State};


pub trait Observer {
    type Event;
}

pub trait Subscriber {
    type Event;
    fn observe(&mut self, event: &Self::Event);
}

pub struct EventDispatcher<Event> {
    subscribers: Vec<ComponentInner<dyn Subscriber<Event = Event>>>,
}

impl<E> EventDispatcher<E> {
    pub fn new() -> Self {
        Self {
            subscribers: vec![],
        }
    }
    pub fn register<S: Subscriber<Event = E> + State + 'static>(&mut self, sub: Component<S>) {
        self.subscribers.push(sub.inner() as ComponentInner<dyn Subscriber<Event = E>>)
    }
    pub fn emit(&self, event: E) {
        self.subscribers.iter().for_each(|sub| {
            sub.borrow_mut().observe(&event)
        })
    }
}