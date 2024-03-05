use xgrid::*;

#[derive(Clone, Copy, Debug)]
struct Div {
    color: [u8; 4],
}
impl Div {
    const UA: u8 = 100;
    const DA: u8 = 255;
}

impl State for Div {
    type Msg = bool;
    type Param = [u8; 3];
    fn init<P: State>(builder: &mut Builder<P>, param: Self::Param) -> Self {
        Self { color: [param[0], param[1], param[2], Self::DA] }
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        self.color[3] =  match msg {
            true => Self::DA,
            false => Self::UA,
        };
        queue.push(UpdateMsg::Frame(FrameMessage {
            color: Some(self.color),
            ..FrameMessage::default()
        }));
    }
}
impl Observer for Div {
    type Event = i32;
}

struct App {
    states: [Component<Div>; 6],
}

impl State for App {
    type Msg = bool;
    type Param = ();
    fn init<P: State>(builder: &mut Builder<P>, _: ()) -> Self {
        let mut g = builder.grid_builder();
        let [yn] = g.heights().add_expanding(Fraction(1)).assign();
        let [x] = g.widths().add(Ratio(1.0)).assign();
        let g = builder.grid(g);
        Self {
            states: [
                builder.frame([255,0,0],g, x, yn),
                builder.frame([0,255,0],g, x, yn),
                builder.frame([0,0,255],g, x, yn),
                builder.frame([255,255,0],g, x, yn),
                builder.frame([255,0,255],g, x, yn),
                builder.frame([0,255,255],g, x, yn),
            ],
        }
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        self.states.iter_mut().for_each(|s| s.update(msg, queue))
    }
}

fn main() {
    xgrid::run::<App>();
}
