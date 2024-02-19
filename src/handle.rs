use std::{marker::PhantomData, num::NonZeroUsize};


#[derive(Clone, Copy)]
pub struct Handle<T> {
    index: NonZeroUsize,
    _t: PhantomData<T>,
}
impl<T> Handle<T> {
    pub (crate) fn new(index: usize) -> Self {
        Self {
            index: NonZeroUsize::try_from(index + 1).unwrap(),
            _t: PhantomData::<T>,
        }
    }
    pub (crate) fn index(&self) -> usize {
        self.index.get() - 1
    }
}
