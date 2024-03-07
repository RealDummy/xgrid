use crate::{units::VUnit, Observer};



pub struct Position {
    x: VUnit,
    y: VUnit,
}
pub enum MouseButton {
    Left,
    Right,
}
pub enum MouseEvent {
    Move(Position),
    Click(MouseButton)
}
pub struct MouseObserver {}
impl Observer for MouseObserver {
    type Event = MouseEvent;
}


struct KeyboardKeyPress {
    pub key: char,
    pub modifiers: winit::keyboard::ModifiersState
}
pub enum KeyboardEvent {
    Press(KeyboardKeyPress),
    Release(KeyboardKeyPress)
}

struct KeyBoardObserver {}
impl Observer for KeyBoardObserver {
    type Event = KeyboardEvent;
}