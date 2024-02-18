use std::{fmt::Debug, iter, vec};

use log::{debug, error, info, warn};

use crate::{frame::{FrameData, FrameRenderer}, units::{Fractiont, UserUnits, VUnit}, BBox, WorldView};


pub enum SpacerType<NameT: Debug + Default + Eq> {
    Unit(UserUnits, NameT),
    Repeat(UserUnits),
}

fn spacer_after_repeat_count(spacer: &GridSpacer) -> usize {
    spacer.iter().skip_while(|s| match s {SpacerType::Repeat(_) => false, _ => true}).count()
}

enum SolveUnits {
    Exact(VUnit),
    Fraction(Fractiont),
}

fn units_solve(u: &UserUnits, pos: VUnit, len: VUnit) -> SolveUnits {
    use SolveUnits::*;
    match u {
        UserUnits::Pixel(p) => {
            Exact(VUnit((*p).into()))
        },
        UserUnits::Ratio(f) => {
            Exact(VUnit((len.0 as f32 * f).round() as i32))
        }
        UserUnits::Zero => Exact(VUnit(0)),
        UserUnits::Fraction(f) => Fraction(*f),
    }
}

struct SpacerSolved {
    pos: VUnit,
    len: VUnit,
}

fn expand_spacer<'a>(spacer: &'a GridSpacer, pos: VUnit, len: VUnit, repeat_count: usize) -> impl Iterator<Item = SpacerSolved> + 'a {
    let iter_res = spacer.iter().map(move |s| {
        match s {
            SpacerType::Unit(u, _) => {
                iter::repeat(u).take(1)
            }
            SpacerType::Repeat(u) => {
                iter::repeat(u).take(repeat_count)
            }
        }
    }).flatten().map(move |u| units_solve(u, pos, len));
    let (total_f, taken_u) = iter_res.clone().fold((0,0),|(a, rest), u| {
        match u {
            SolveUnits::Fraction(f) => (a + f, rest),
            SolveUnits::Exact(v) => (a, rest + v.0),
        }
    });
    let units_remaining = len.0 - taken_u;
    let mut curr_pos = 0;
    let iter_res = iter_res.map(move |u| {
        SpacerSolved {
            pos: VUnit(curr_pos),
            len: match u {
                SolveUnits::Exact(u) => {
                    curr_pos += u.0;
                    u
                }
                SolveUnits::Fraction(f) => {
                    let u = {
                        VUnit(((f as f32 / total_f as f32) * units_remaining as f32).round() as i32)
                    };
                    curr_pos += u.0;
                    u
                }
            }
        }
    });
    iter_res
}


pub type GridSpacer<NameT> = Vec<SpacerType<NameT>>;

pub enum GridRepeatDir {
    X,
    Y,
}


pub struct Grid<T: Debug + Default + Eq> {
    pub handles: Vec<Option<usize>>,
    pub x_spacer: GridSpacer<T>,
    pub y_spacer: GridSpacer<T>,
    pub repeat: Option<GridRepeatDir>,
    pub parent_frame_handle: usize,
    outer_vec: Vec<SpacerSolved>,
    inner_vec: Vec<SpacerSolved>,
}   

impl<T: Debug + Eq + Default> Grid<T> {
    pub fn new(parent_frame_handle: usize, x_spacer: GridSpacer<T>, y_spacer: GridSpacer<T>, repeat: Option<GridRepeatDir> ) -> Self {
        Self {
            x_spacer,
            y_spacer,
            handles: vec![],
            repeat,
            outer_vec: vec![],
            inner_vec: vec![],
            parent_frame_handle,
        }
    }
    
    pub fn update(&mut self, frames: &mut FrameRenderer) {
        use GridRepeatDir::*;
        const OUT_OFFSET: usize = 1;
        const IN_OFFSET: usize = 0;
        let bounds = {
            let BBox { x, y, w, h } = frames.get(self.parent_frame_handle).data;
            [x,y,w,h]
        };
        let (outer, inner, bounds_offset) =  match self.repeat {
            Some(X) => (&self.x_spacer, &self.y_spacer, IN_OFFSET),
            Some(Y) => (&self.y_spacer, &self.x_spacer, OUT_OFFSET),
            None => (&self.y_spacer, &self.x_spacer, OUT_OFFSET),
        };
        let (outer_offset, inner_offset) = (bounds_offset, 1-bounds_offset);
        let repeat_count = (self.handles.len() / inner.len()).checked_sub(outer.len() - 1).unwrap_or(0);
        let outer_spacer = expand_spacer(outer, bounds[outer_offset], bounds[2 + outer_offset], repeat_count);
        let inner_spacer = expand_spacer(inner, bounds[inner_offset], bounds[2 + inner_offset], 0);
        self.outer_vec.clear();
        self.inner_vec.clear();
        self.outer_vec.extend(outer_spacer);
        self.inner_vec.extend(inner_spacer);
        let (xvec, yvec) = match bounds_offset {
            OUT_OFFSET => (&self.inner_vec,&self.outer_vec),
            IN_OFFSET | _ => (&self.outer_vec,&self.inner_vec), 
        };
        self.handles.iter().enumerate().for_each(|(i, h)|{
            let Some(handle) = h else {
                return;
            };
            let (x,y) = match outer_offset {
                OUT_OFFSET => (i/self.y_spacer.len(), i%self.y_spacer.len()),
                _ => (i%self.x_spacer.len(), i/self.x_spacer.len())
            };
            let SpacerSolved {
                pos: x,
                len: w,
            } = xvec[x];
            let SpacerSolved {
                pos: y,
                len: h,
            } = yvec[y];
            
            frames.update(*handle, BBox{x,y,w,h});

        })
        

    }

} 
