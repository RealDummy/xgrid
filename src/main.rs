use xgrid::*;

#[derive(Clone, Copy, Debug)]
struct Div {
    i: i32,
    color: [u8; 4],
}
impl Div {
    const UA: u8 = 100;
    const DA: u8 = 255;
}

impl State for Div {
    type Msg = bool;
    type Param = (i32, EventDispatcher<i32>);
    fn init<P: State>(_builder: &mut Builder<P>, param: &Self::Param) -> Self {
        Self {
            color: [255, 0, 0, Self::DA],
            i: param.0,
        }
    }
    fn after_init(
        component: &Component<Self>,
        _system_events: &mut SystemEvents,
        param: &Self::Param,
    ) {
        param.1.register(component);
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        self.color[3] = match msg {
            true => Self::DA,
            false => Self::UA,
        };
        queue.push(UpdateMsg::Frame(FrameMessage {
            color: Some(self.color),
            ..FrameMessage::default()
        }));
    }
}
impl Subscriber<i32> for Div {
    fn observe(&mut self, event: &i32, _queue: &UpdateQueue) {
        if event % self.i == 0 {
            self.color[0..3].rotate_left(1);
        }
    }
}

struct App {
    states: Vec<Component<Div>>,
    state_event: EventDispatcher<i32>,
    i: i32,
}

impl State for App {
    type Msg = bool;
    type Param = ();
    fn init<P: State>(builder: &mut Builder<P>, _: &()) -> Self {
        let mut g = builder.grid_builder();
        let [_yn] = g.heights().add_expanding(Fraction(1)).assign();
        g.widths()
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .add(Fraction(1))
            .build();
        let g = builder.grid(g);
        let state_event = builder.event_dispatcher();
        Self {
            states: (1..=128)
                .into_iter()
                .map(|i| builder.frame((i, state_event.clone()), g, None, None))
                .collect(),
            state_event,
            i: 0,
        }
    }
    fn after_init(
        component: &Component<Self>,
        system_events: &mut SystemEvents,
        _param: &Self::Param,
    ) {
        system_events.add_mouse_observer(component)
    }
    fn update(&mut self, msg: Self::Msg, queue: &UpdateQueue) {
        self.states.iter_mut().for_each(|s| s.update(msg, queue))
    }
}
impl Subscriber<MouseEvent> for App {
    fn observe(&mut self, event: &MouseEvent, queue: &UpdateQueue) {
        match event {
            MouseEvent::Click(MouseButton::Left(ButtonState::Pressed)) => {
                self.states.iter_mut().for_each(|c| c.update(true, queue));
            }
            MouseEvent::Click(MouseButton::Left(ButtonState::Released)) => {
                self.states
                    .iter_mut()
                    .for_each(|c: &mut Component<Div>| c.update(false, queue));
                self.state_event.emit(self.i);
                self.i += 1;
            }
            _ => (),
        }
    }
}

fn main() {
    xgrid::run::<App>();
}
