use log::debug;
use xgrid::*;




#[derive(Clone, Copy, Debug)]
struct Div {
    down: bool,
}
impl Div {
    const DC: [u8; 4] = [255;4];
    const UC: [u8; 4] = [0; 4];
}

impl Update for Div {
    type Msg = bool;
    fn init(frame: FrameHandle, manager: &mut UpdateManager) -> Self {
        Self {
            down: false,
        }
    }
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager) {
        manager.update_frame_color(frame, if self.down {Div::DC} else {Div::UC} )
    }
    fn update(&mut self, msg: Self::Msg, frame: FrameHandle, manager: &mut UpdateManager) -> bool{
        println!("{:?}", frame);
        return {
            let res = msg != self.down;
            self.down = msg;
            res
        }
   }
}
struct App {
     states: [ComponentHandle<Div>; 6],
}

impl Update for App {
    type Msg = Interaction;
    fn init(frame: FrameHandle, manager: &mut UpdateManager) -> Self {
        let mut g = manager.create_grid_in(frame);
        let [x1, x2, x3] = g
            .widths()
            .add(Pixel(100))
            .add(Ratio(0.2))
            .add_expanding(Fraction(1))
            .assign();
        let [y1, _y2, y3] = g
            .heights()
            .add(Pixel(100))
            .add(Fraction(1))
            .add(Pixel(100))
            .assign();

        let g = g.build(manager);
        App {
            states: [
                manager.add_frame(g, x1, y1),
                manager.add_frame( g,x2, y3),
                manager.add_frame(g, x3, None),
                manager.add_frame(g, x3, None),
                manager.add_frame(g, x3, None),
                manager.add_frame(g, x3, y3),
            ],
        } 
    }
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager) {

    }
    fn update(&mut self, msg: Self::Msg, frame: FrameHandle, manager: &mut UpdateManager) -> bool {
        match msg {
            Interaction::Click( down ) => {
                self.states.iter().for_each(|div| {
                    div.update(down, frame, manager)
                });
            },
            _ => (),
        };
        false
    }
}

fn main() {
    pollster::block_on(xgrid::run::<App>());
}
