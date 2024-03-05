use std::marker::PhantomData;

use crate::{
    frame::FrameHandle,
    handle::{FallableHandleLike, Handle, HandleLike},
    units::UserUnits,
};

use super::{data::{GridData, GridExpandDir}, GridHandle};

#[derive(Clone, Debug)]
pub enum SpacerUnit {
    Unit(UserUnits),
    Repeat(UserUnits),
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Width {}
#[derive(Clone, Copy, Default, Debug)]
pub struct Height {}

pub type XName = Handle<Width>;
pub type YName = Handle<Height>;

pub trait GridDir: Copy + Clone + Default {
    fn dir() -> GridExpandDir;
    fn new(i: Option<usize>) -> Self;
}

impl GridDir for Option<XName> {
    fn dir() -> GridExpandDir {
        GridExpandDir::X
    }
    fn new(i: Option<usize>) -> Self {
        match i {
            Some(i) => Some(Handle::new(i)),
            None => None,
        }
    }
}
impl GridDir for Option<YName> {
    fn dir() -> GridExpandDir {
        GridExpandDir::Y
    }
    fn new(i: Option<usize>) -> Self {
        match i {
            Some(i) => Some(Handle::new(i)),
            None => None,
        }
    }
}

pub(crate) type GridSpacer = Vec<SpacerUnit>;

#[derive(Debug)]
pub struct GridBuilder {
    spacers: [GridSpacer; 2],
    expands: Option<GridExpandDir>,
    parent: FrameHandle,
}


pub struct SpacerBuilder<'b, const EXPANDS: bool, T: GridDir + FallableHandleLike> {
    grid_builder: &'b mut GridBuilder,
    spacer: GridSpacer,
    _dir: PhantomData<T>,
}

impl<'b, const EXPANDS: bool, T: GridDir + FallableHandleLike> SpacerBuilder<'b, EXPANDS, T> {
    fn new(grid_builder: &'b mut GridBuilder) -> Self {
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
    
    pub fn build(self) {
        self.grid_builder.spacers[match T::dir() {
            GridExpandDir::X => 0,
            _ => 1,
        }] = self.spacer;
    }

    pub fn assign<const N: usize>(self) -> [T; N] {
        let mut res = [T::default(); N];
        for (i, (var, _)) in res.iter_mut().zip(self.spacer.iter()).enumerate() {
            *var = T::new(Some(i));
        }
        self.grid_builder.expands = match &self.grid_builder.expands {
            None => Some(T::dir()),
            _ if EXPANDS => {
                panic!("grid's can only have one expanding direction")
            }
            n => *n,
        };
        self.build();
        res
    }
}

impl<'b, T: GridDir + FallableHandleLike> SpacerBuilder<'b, false, T> {
    pub fn add_expanding(mut self: Self, u: UserUnits) -> SpacerBuilder<'b, true, T> {
        self.spacer.push(SpacerUnit::Repeat(u.clone()));
        SpacerBuilder {
            spacer: self.spacer,
            grid_builder: self.grid_builder,
            _dir: PhantomData,
        }
    }
}

pub type WidthSpacerBuilder<'a, const EXPANDS: bool> = SpacerBuilder<'a, EXPANDS, Option<XName>>;
pub type HeightSpacerBuilder<'a, const EXPANDS: bool> = SpacerBuilder<'a, EXPANDS, Option<YName>>;

impl GridBuilder {
    pub(crate) fn new(parent: FrameHandle) -> GridBuilder {
        GridBuilder {
            spacers: [GridSpacer::new(), GridSpacer::new()],
            expands: None,
            parent,
        }
    }
    pub(crate) fn parent(&self) -> FrameHandle {
        self.parent
    }
    pub fn widths(&mut self) -> WidthSpacerBuilder<false> {
        WidthSpacerBuilder::new(self)
    }
    pub fn heights(&mut self) -> HeightSpacerBuilder<false> {
        HeightSpacerBuilder::new(self)
    }
    pub fn build(self) -> GridData {
        let [x_spacer, y_spacer] = self.spacers;
        GridData::new(self.parent, x_spacer, y_spacer, self.expands)
    }
}
