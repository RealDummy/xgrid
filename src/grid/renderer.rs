use std::iter;

use log::error;

use crate::frame::FrameHandle;
use crate::handle::HandleLike;
use crate::units::{Fractiont, UserUnits};
use crate::{handle::Handle, FrameRenderer};
use crate::{BBox, VUnit};

use super::{Grid, GridSpacer, SpacerUnit, XName, YName};
use crate::handle::FallableHandleLike;

pub struct GridRenderer {
    data: Vec<Grid>,
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
        frame_renderer: &mut FrameRenderer,
        grid_handle: GridHandle,
        frame_handle: FrameHandle,
        x: XName,
        y: YName,
    ) {
        match self.data[grid_handle.index()].add_frame(frame_handle, x, y) {
            Ok(()) => (),
            Err(()) => error!("couldn't add {} to grid at x:{:?} y:{:?}",frame_handle.index(), x.index(), y.index()),
        }
        self.data[grid_handle.index()].update(frame_renderer);
    }
    pub fn add(&mut self, g: Grid) -> GridHandle {
        self.data.push(g);
        return GridHandle::new(self.data.len() - 1);
    }
    pub fn get_parent_handle(&self, grid: GridHandle) -> FrameHandle {
        self.data[grid.index()].parent()
    }
}
