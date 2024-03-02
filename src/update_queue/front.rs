use crate::manager::{BBox, MarginBox, Rect};

pub struct UpdateQueue {
    
}

pub enum Bounds {
    Rel(BBox),
    Abs(BBox),
}

pub enum Update {
    Bounds(Bounds),
    Margin(MarginBox),
    Color([u8; 4]),
}


impl UpdateQueue {

    pub fn push(&self, update: Update) -> &Self {
        
        self
    }
}