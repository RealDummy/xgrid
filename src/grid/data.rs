use std::{clone, fmt::Debug, iter, marker::PhantomData, vec};

use log::{debug, error, info, warn};

use crate::{frame::{FrameData, FrameHandle, FrameRenderer}, handle::Handle, manager::UpdateManager, units::{Fractiont, UserUnits, VUnit}, BBox, WorldView};

use super::{GridHandle, GridRenderer};

#[derive(Clone)]
pub enum SpacerUnit {
    Unit(UserUnits),
    Repeat(UserUnits),
}

const X: usize = 0;
const Y: usize = 1;

#[derive(Clone, Copy, Default)]
pub struct Width {}
#[derive(Clone, Copy, Default)]
pub struct Height {}

pub type XName = Option<Handle<Width>>;
pub type YName = Option<Handle<Height>>;

pub trait Dir: Copy + Clone + Default {
    fn item() -> GridExpandDir;
}

impl Dir for XName {
    fn item() -> GridExpandDir {
        GridExpandDir::X
    }
}
impl Dir for YName {
    fn item() -> GridExpandDir {
        GridExpandDir::Y
    }
}

type GridSpacer = Vec<SpacerUnit>;

pub struct GridBuilder<const XEXPANDS: bool, const YEXPANDS: bool> {
    spacers: [GridSpacer; 2],
    expands: Option<GridExpandDir>,
    parent: FrameHandle,
}



pub struct SpacerBuilder<'b, const XEXPANDS: bool, const YEXPANDS: bool, T: Dir> {
    grid_builder: &'b mut GridBuilder<XEXPANDS, YEXPANDS>,
    spacer: GridSpacer,
    _dir: PhantomData<T>,
}

impl<'b, const XEXPANDS: bool,const YEXPANDS: bool, T: Dir> SpacerBuilder<'b, XEXPANDS, YEXPANDS, T> {
    fn new(grid_builder: &'b mut GridBuilder<XEXPANDS, YEXPANDS>)-> Self {
        Self {
            grid_builder: grid_builder,
            spacer: GridSpacer::new(),
            _dir: PhantomData,
        }
    }
    pub fn add(mut self, u: UserUnits) -> Self {
        self.spacer.push(SpacerUnit::Unit(u.clone()));
        self
    }

    pub fn assign<const N: usize>(self) -> [Option<Handle<T>>; N] {
        let mut res = [None; N];
        for (i, (var, _)) in res.iter_mut().zip(self.spacer.iter()).enumerate() {
            *var = Some(Handle::new(i));
        }
        self.grid_builder.spacers[match T::item() {GridExpandDir::X => 0, _=>1}] = self.spacer;
        self.grid_builder.expands = Some(T::item());
        res
    }
}

impl<'b> SpacerBuilder<'b, false, false, XName> {
    pub fn add_expanding(mut self, u: UserUnits) -> SpacerBuilder<'b, true, false, XName> {
        self.spacer.push(SpacerUnit::Repeat(u.clone()));
        self
    }
}

impl<'b> SpacerBuilder<'b, false, false, YName> {
    pub fn add_expanding(mut self, u: UserUnits) -> SpacerBuilder<'b, false, true, YName> {
        self.spacer.push(SpacerUnit::Repeat(u.clone()));
        SpacerBuilder { 
            grid_builder: self.grid_builder,
            spacer: self.spacer,
            _dir, PhantomData
        }
    }
}

pub type WidthSpacerBuilder<'a, const XEXPANDS: bool, const YEXPANDS:bool> = SpacerBuilder<'a, XEXPANDS, YEXPANDS, XName>;
pub type HeightSpacerBuilder<'a, const XEXPANDS: bool, const YEXPANDS:bool> = SpacerBuilder<'a, XEXPANDS, YEXPANDS, YName>;


impl<const XEXPANDS: bool, const YEXPANDS: bool> GridBuilder<XEXPANDS, YEXPANDS> {
    pub fn new(parent: FrameHandle)-> GridBuilder<false, false> {
        GridBuilder::<false, false> {
            spacers: [
                GridSpacer::new(),
                GridSpacer::new(),
            ],
            expands: None,
            parent,
        }
    }
    pub fn widths(&mut self) -> WidthSpacerBuilder<XEXPANDS, YEXPANDS> {
        WidthSpacerBuilder::new(self)
    }
    pub fn heights(&mut self) -> HeightSpacerBuilder<XEXPANDS, YEXPANDS> {
        HeightSpacerBuilder::new(self)
    }
    pub fn build(self, manager: &mut UpdateManager) -> GridHandle {
        let [x_spacer, y_spacer] = self.spacers;
        manager.add_grid(Grid::new(
        self.parent, 
                x_spacer,
                y_spacer,
                self.expands,
            )
        )
    }
}


enum SolveUnits {
    Exact(VUnit),
    Fraction(Fractiont),
}

fn units_solve(u: UserUnits, len: VUnit) -> SolveUnits {
    use SolveUnits::*;
    match u {
        UserUnits::Pixel(p) => {
            Exact(p.into())
        }
        UserUnits::Ratio(f) => {
            Exact(((len.pix() * f).round() as i32).into())
        }
        UserUnits::Zero => Exact(0.into()),
        UserUnits::Fraction(f) => Fraction(f),
    }
}

struct SpacerSolved {
    pos: VUnit,
    len: VUnit,
}

fn expand_spacer<'a>(spacer: &'a GridSpacer, pos: VUnit, len: VUnit, repeat_count: usize) -> impl Iterator<Item = SpacerSolved> + 'a {
    let iter_res = spacer.iter().map(move |s| {
        match s {
            SpacerUnit::Unit(u) => {
                iter::repeat(u).take(1)
            }
            SpacerUnit::Repeat(u) => {
                iter::repeat(u).take(repeat_count + 1)
            }
        }
    }).flatten().map(move |&u| units_solve(u, len));
    let (total_f, taken_u) = iter_res.clone().fold((0, 0.into()),|(a, rest), u| {
        match u {
            SolveUnits::Fraction(f) => (a + f, rest),
            SolveUnits::Exact(v) => (a, rest + v),
        }
    });
    let units_remaining = len - taken_u;
    let mut curr_pos = pos;
    let iter_res = iter_res.map(move |u| {
        SpacerSolved {
            pos: curr_pos,
            len: match u {
                SolveUnits::Exact(u) => {
                    curr_pos += u;
                    u
                }
                SolveUnits::Fraction(f) => {
                    let u = ((f as i32) * units_remaining) / (total_f as i32);
                    curr_pos += u;
                    u
                }
            }
        }
    });
    iter_res
}

pub enum GridExpandDir {
    X,
    Y,
}


pub struct Grid {
    pub handles: Vec<Option<FrameHandle>>,
    pub x_spacer: GridSpacer,
    pub y_spacer: GridSpacer,
    pub expand_dir: GridExpandDir,
    pub parent_frame_handle: FrameHandle,
    outer_vec: Vec<SpacerSolved>,
    inner_vec: Vec<SpacerSolved>,
}   

impl Grid {
    pub fn new(parent_frame_handle: FrameHandle, x_spacer: GridSpacer, y_spacer: GridSpacer, expand_dir: Option<GridExpandDir> ) -> Self {
        Self {
            x_spacer,
            y_spacer,
            handles: vec![],
            expand_dir: expand_dir.unwrap_or(GridExpandDir::Y),
            outer_vec: vec![],
            inner_vec: vec![],
            parent_frame_handle,
        }
    }
    
    pub fn update(&mut self, frames: &mut FrameRenderer) {
        const OUT_OFFSET: usize = 1;
        const IN_OFFSET: usize = 0;
        let bounds = {
            let BBox { x, y, w, h } = frames.get(&self.parent_frame_handle).data;
            [x,y,w,h]
        };
        let (outer, inner, bounds_offset) =  match self.expand_dir {
            GridExpandDir::X => (&self.x_spacer, &self.y_spacer, IN_OFFSET),
            GridExpandDir::Y => (&self.y_spacer, &self.x_spacer, OUT_OFFSET),
        };
        let (outer_offset, inner_offset) = (bounds_offset, 1-bounds_offset);
        let repeat_count = (self.handles.len() / inner.len()).checked_sub(outer.len() - 1).unwrap_or(0);
        let outer_spacer = expand_spacer(outer, bounds[outer_offset], bounds[2 + outer_offset], repeat_count);
        let inner_spacer = expand_spacer(inner, bounds[inner_offset], bounds[2 + inner_offset], repeat_count);
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
                OUT_OFFSET => (i/yvec.len(), i%yvec.len()),
                _ => (i%xvec.len(), i/xvec.len())
            };
            let SpacerSolved {
                pos: x,
                len: w,
            } = xvec[x];
            let SpacerSolved {
                pos: y,
                len: h,
            } = yvec[y];
            frames.update(handle, &BBox{x,y,w,h});

        })
    }
    pub fn add_frame(&mut self, handle: FrameHandle) {
        self.handles.push(Some(handle))
    }

} 
