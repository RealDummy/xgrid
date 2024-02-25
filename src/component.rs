use std::iter::Iterator;

use crate::message::FrameMessage;

trait Component {
    type UpdateMessage;
    fn build(&self) -> dyn Iterator<Item = Box<dyn Component<UpdateMessage = Self::UpdateMessage>>>;
    fn receive(&self, send: dyn Fn(Box<dyn FrameMessage>));
}