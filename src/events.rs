use crate::{units::VUnit};



pub struct Position {
    x: VUnit,
    y: VUnit,
}
pub enum ButtonState {
    Pressed,
    Released,
}
impl ButtonState {
    pub fn pressed(&self) -> bool {
        matches!(self, Self::Pressed)
    }
}
pub enum MouseButton {
    Left(ButtonState),
    Right(ButtonState),
}
pub enum MouseEvent {
    Move(Position),
    Click(MouseButton)
}

pub struct KeyboardKey {
    pub key: char,
    pub modifiers: winit::keyboard::ModifiersState
}
pub enum KeyboardEvent {
    Press(KeyboardKey),
    Release(KeyboardKey)
}