use std::{marker::PhantomData, num::NonZeroUsize};

#[derive(Clone, Copy, Debug)]
pub struct Handle<T> {
    index: NonZeroUsize,
    _t: PhantomData<T>,
}

pub trait FallableHandleLike {
    fn index(&self) -> Option<usize>;
}

impl<T: HandleLike> FallableHandleLike for Option<T> {
    fn index(&self) -> Option<usize> {
        match self {
            None => None,
            Some(h) => Some(T::index(h)),
        }
    }
}

pub trait HandleLike {
    fn new(i: usize) -> Self;
    fn index(&self) -> usize;
}

impl<T> HandleLike for Handle<T> {
    fn new(i: usize) -> Self {
        Self {
            index: NonZeroUsize::try_from(i + 1).unwrap(),
            _t: PhantomData,
        }
    }
    fn index(&self) -> usize {
        self.index.get() - 1
    }
}
