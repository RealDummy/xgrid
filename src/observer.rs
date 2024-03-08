use std::{cell::RefCell, rc::Rc};

use crate::{component::{ComponentInner, ComponentType}, update_queue::{self, front}, Component, State, UpdateQueue};


pub trait Subscriber<Event> {
    fn observe(&mut self, event: &Event, queue: &UpdateQueue);
}

#[derive(Clone)]
pub struct EventDispatcher<Event> {
    subscribers: Rc<RefCell<Vec<(ComponentInner<dyn Subscriber<Event>>, ComponentType)>>>,
    queue: front::UpdateQueue,
}

impl<E> EventDispatcher<E> {
    pub fn new(queue: &front::UpdateQueue) -> Self {
        Self {
            subscribers: Rc::new(RefCell::new(vec![])),
            queue: queue.clone(),
        }
    }
    pub fn register<S: Subscriber<E> + State + 'static>(&self, sub: &Component<S>) {
        self.subscribers.borrow_mut().push((sub.inner().clone() as ComponentInner<dyn Subscriber<E>>, sub.handle.clone()));
    }
    pub fn emit(&self, event: E) {
        self.subscribers.borrow().iter().for_each(|(sub, handle)| {
            sub.borrow_mut().observe(&event, &UpdateQueue::from_base(&self.queue, handle.clone()))
        })
    }
}