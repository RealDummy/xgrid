use std::{iter, marker::PhantomData};

use crate::{frame::FrameHandle, handle::{FallableHandleLike, Handle, HandleLike}, manager::UpdateManager, units::{Fractiont, UserUnits, VUnit}};

use super::{data::GridExpandDir, GridHandle};

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
    fn dir() -> GridExpandDir;
    fn new(i: Option<usize>) -> Self;
}

impl Dir for XName {
    fn dir() -> GridExpandDir {
        GridExpandDir::X
    }
    fn new(i: Option<usize>) -> Self {
        match i {
            Some(i) => Some(Handle::new(i)),
            None => None
        }
    }
}
impl Dir for YName {
    fn dir() -> GridExpandDir {
        GridExpandDir::Y
    }
    fn new(i: Option<usize>) -> Self {
        match i {
            Some(i) => Some(Handle::new(i)),
            None => None
        }
    }
}

pub(crate) type GridSpacer = Vec<SpacerUnit>;

pub struct GridBuilder {
    spacers: [GridSpacer; 2],
    expands: Option<GridExpandDir>,
    parent: FrameHandle,
}


pub struct SpacerBuilder<'b, const EXPANDS: bool, T: Dir + FallableHandleLike> {
    grid_builder: &'b mut GridBuilder,
    spacer: GridSpacer,
    _dir: PhantomData<T>,
}

impl<'b, const EXPANDS: bool, T: Dir + FallableHandleLike> SpacerBuilder<'b, EXPANDS, T> {
    fn new(grid_builder: &'b mut GridBuilder)-> Self {
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

    pub fn assign<const N: usize>(self) -> [T; N] {
        let mut res = [T::default(); N];
        for (i, (var, _)) in res.iter_mut().zip(self.spacer.iter()).enumerate() {
            *var = T::new(Some(i));
        }
        self.grid_builder.expands = match &self.grid_builder.expands {
            None => Some(T::dir()),
            _ if EXPANDS => {panic!("grid's can only have one expanding direction")},
            n => *n,
        };
        self.grid_builder.spacers[match T::dir() {GridExpandDir::X => 0, _=>1}] = self.spacer;
        res
    }
}

impl<'b, T:Dir + FallableHandleLike> SpacerBuilder<'b, false, T> {
    pub fn add_expanding(mut self: Self, u: UserUnits) -> SpacerBuilder<'b, true, T> {
        self.spacer.push(SpacerUnit::Repeat(u.clone()));
        SpacerBuilder {
            spacer: self.spacer,
            grid_builder: self.grid_builder,
            _dir: PhantomData,
        }
    }
}

pub type WidthSpacerBuilder<'a, const EXPANDS: bool> = SpacerBuilder<'a, EXPANDS, XName>;
pub type HeightSpacerBuilder<'a, const EXPANDS: bool> = SpacerBuilder<'a, EXPANDS, YName>;


impl GridBuilder {
    pub fn new(parent: FrameHandle)-> GridBuilder {
        GridBuilder {
            spacers: [
                GridSpacer::new(),
                GridSpacer::new(),
            ],
            expands: None,
            parent,
        }
    }
    pub fn widths(&mut self) -> WidthSpacerBuilder<false> {
        WidthSpacerBuilder::new(self)
    }
    pub fn heights(&mut self) -> HeightSpacerBuilder<false> {
        HeightSpacerBuilder::new(self)
    }
    pub fn build(self, manager: &mut UpdateManager) -> GridHandle {
        let [x_spacer, y_spacer] = self.spacers;
        manager.add_grid(crate::grid::Grid::new(
        self.parent, 
                x_spacer,
                y_spacer,
                self.expands,
            )
        )
    }
}

