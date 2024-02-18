use std::ops::Index;

use super::Grid;
use crate::frame::FrameHandle;
use crate::{FrameRenderer, MarginBox, handle::Handle};
use crate::{BBox, frame::FrameData, VUnit};
use log::debug;

pub struct GridRenderer {
    data: Vec<Grid>,
}

pub type GridHandle = Handle<Grid>;

impl GridRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, world_view_layout: &wgpu::BindGroupLayout) -> Self {
        Self {
            data: vec![],
        }
    }
    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        ()
    }
    pub fn render<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp> ) {
        ()
    }
    pub fn update(&mut self, grid_handle: usize, bounds: &BBox, frame_renderer: &mut FrameRenderer) {
        self.data[grid_handle].update(frame_renderer);
    }
    pub fn add_frame(&mut self, grid_handle: GridHandle, frame_handle: FrameHandle) {
        self.data[grid_handle.index()].handles.push(Some(frame_handle));
    }
    pub fn add(&mut self, g: Grid) -> GridHandle {
        self.data.push(g);
        return GridHandle::new(self.data.len() - 1);
    }
}