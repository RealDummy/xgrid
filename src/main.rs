use xgrid::*;

#[derive(Clone, Copy, Debug)]
struct Div {
    color: [u8; 4],
}
impl Update for Div {
    type Msg = bool;
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager) {
        
    }
    fn update(&mut self, msg: Self::Msg) {
        match msg {
            true => {
                self.color[0] = 0;
            },
            false => {
                self.color[0] = 255;
            }
        }
   }
}
struct App {
     states: [Div; 6],
}

impl Update for App {
    type Msg = Interaction;
    fn build(&self, frame: FrameHandle, manager: &mut UpdateManager) {
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
        let c1 = manager.add_frame(self.states[0],g, x1, y1);
        let c2 = manager.add_frame( self.states[1],g,x2, y3);
        let c3 = manager.add_frame(self.states[2],g, x3, None);
        let c4 = manager.add_frame(self.states[3],g, x3, None);
        let c5 = manager.add_frame(self.states[4],g, x3, None);
        let c6 = manager.add_frame(self.states[5],g, x3, y3);
    }
    fn update(&mut self, msg: Self::Msg) {
        match msg {
            Interaction::Click(down) => {
                self.states.iter_mut().for_each(|div| {
                    div.update(down)
                })
            },
            _ => (),
        }
    }
}

fn main() {
    let app = App {
        states: [Div{color: [255;4]}; 6],
    };
    pollster::block_on(xgrid::run(app));
}
