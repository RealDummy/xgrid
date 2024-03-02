
use xgrid::*;

#[derive(Clone, Copy, Debug)]
struct Div {
    down: bool,
}
impl Div {
    const UC: [u8; 4] = [100; 4];
    const DC: [u8; 4] = [50; 4];
}

impl Update for Div {
    type Msg = bool;
    fn init() -> Self {
        Self { down: false }
    }
    fn build(&self) {
        // manager.get_frame_data(frame ).color = if self.down {Div::DC} else {Div::UC}
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        return {
            let res = msg != self.down;
            self.down = msg;
            res
        };
    }
}
struct App {
    states: [Div; 6],
}

impl Update for App {
    type Msg = Interaction;
    fn init() -> Self {
        todo!()
    }
    fn build(&self) {}
    fn update(&mut self, _msg: Self::Msg) -> bool {
        todo!()
    }
}

fn main() {
    xgrid::run::<App>();
}
