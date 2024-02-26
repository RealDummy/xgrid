use std::{collections::HashMap, fmt::Debug, iter::{self, Iterator}, rc::Rc};

use crate::{frame::FrameHandle};
// trait Message {
//     fn apply(&self, target: &mut dyn Component);
// }

trait Message {

}

trait Update {
    type Msg: Debug;
    fn update(&self, msg: Self::Msg);
}



trait Frame {
    
}


impl<S: Update> Update for Component<S> {
    type Msg = S::Msg;
    fn update(&self, msg: Self::Msg) {
        self.state.update(msg);
    }
}

struct Component<S: Update> {
    state: S,
    handle: FrameHandle,
}

impl<S: Update> Frame for Component<S> {

}
