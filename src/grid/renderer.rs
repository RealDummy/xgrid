use std::iter;

use crate::frame::FrameHandle;
use crate::handle::HandleLike;
use crate::units::{Fractiont, UserUnits};
use crate::{handle::Handle, FrameRenderer};
use crate::{BBox, VUnit};

use super::{Grid, GridSpacer, SpacerUnit, XName, YName};

pub struct GridRenderer {
    data: Vec<Grid>,
}

pub struct SpacerSolved {
    pub pos: VUnit,
    pub len: VUnit,
}

enum SolveUnits {
    Exact(VUnit),
    Fraction(Fractiont),
}

fn units_solve(u: UserUnits, len: VUnit) -> SolveUnits {
    use SolveUnits::*;
    match u {
        UserUnits::Pixel(p) => Exact(p.into()),
        UserUnits::Ratio(f) => Exact(((len.pix() * f).round() as i32).into()),
        UserUnits::Zero => Exact(0.into()),
        UserUnits::Fraction(f) => Fraction(f),
    }
}

pub(crate) fn expand_spacer<'a>(
    spacer: &'a GridSpacer,
    pos: VUnit,
    len: VUnit,
    repeat_count: usize,
) -> impl Iterator<Item = SpacerSolved> + 'a {
    let iter_res = spacer
        .iter()
        .map(move |s| match s {
            SpacerUnit::Unit(u) => iter::repeat(u).take(1),
            SpacerUnit::Repeat(u) => iter::repeat(u).take(repeat_count + 1),
        })
        .flatten()
        .map(move |&u| units_solve(u, len));
    let (total_f, taken_u) = iter_res
        .clone()
        .fold((0, 0.into()), |(a, rest), u| match u {
            SolveUnits::Fraction(f) => (a + f, rest),
            SolveUnits::Exact(v) => (a, rest + v),
        });
    let units_remaining = (len - taken_u).max(0.into());
    let mut curr_pos = pos;
    let iter_res = iter_res.map(move |u| SpacerSolved {
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
        },
    });
    iter_res
}

#[derive(Clone, Copy, Default)]
pub struct GridT {}

pub type GridHandle = Handle<GridT>;

impl GridRenderer {
    pub fn new(_device: &wgpu::Device, _config: &wgpu::SurfaceConfiguration) -> Self {
        Self { data: vec![] }
    }
    pub fn prepare(&mut self, _queue: &wgpu::Queue) {
        ()
    }
    pub fn render<'rp>(&'rp self, _render_pass: &mut wgpu::RenderPass<'rp>) {
        ()
    }
    pub fn update(
        &mut self,
        grid_handle: GridHandle,
        _bounds: &BBox,
        frame_renderer: &mut FrameRenderer,
    ) {
        self.data[grid_handle.index()].update(frame_renderer);
    }
    pub fn add_frame(
        &mut self,
        grid_handle: GridHandle,
        frame_handle: FrameHandle,
        x: XName,
        y: YName,
    ) {
        self.data[grid_handle.index()].add_frame(frame_handle, x, y);
    }
    pub fn add(&mut self, g: Grid) -> GridHandle {
        self.data.push(g);
        return GridHandle::new(self.data.len() - 1);
    }
    pub fn get_parent_handle(&self, grid: GridHandle) -> FrameHandle {
        //self.data[grid.index()].
        todo!()
    }
}
