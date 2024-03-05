
use log::debug;
use xgrid::*;

#[derive(Clone, Copy, Debug)]
struct Div {
    down: bool,
}
impl Div {
    const UC: [u8; 4] = [100; 4];
    const DC: [u8; 4] = [50; 4];
}

impl State for Div {
    type Msg = bool;
    fn init(builder: &mut Builder) -> Self {
        Self {
            down: false,
        }
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        if self.down != msg {
            debug!("{}", self.down);
            queue.push(UpdateMsg::Frame(FrameMessage{
                color: Some(match msg {
                true => Self::DC,
                false => Self::UC,
                }),
                ..FrameMessage::default()
            }));
            self.down = msg;
        }
    }
}
struct App {
    states: [Component<Div>; 6],
}

impl State for App {
    type Msg = bool;
    fn init(builder: &mut Builder) -> Self {
        let mut g = builder.grid_builder();
        let [yn] = g.heights()
        .add_expanding(Fraction(1))
        .assign();
        let [x] = g.widths()
        .add(Ratio(1.0))
        .assign();
        let g = builder.grid(g);
        Self {
            states: [
                builder.frame(g, x, yn),
                builder.frame(g, x, yn),
                builder.frame(g, x, yn),
                builder.frame(g, x, yn),
                builder.frame(g, x, yn),
                builder.frame(g, x, yn),
            ]
        }
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        self.states.iter_mut().for_each(|s| s.update(msg, queue))
    }
}

fn main() {
    xgrid::run::<App>();
}
