use bytemuck::Zeroable;
use log::debug;
use wgpu::RenderPass;

use crate::{frame::{FrameData, FrameHandle}, grid::{Grid, GridHandle, GridRepeatDir, GridSpacer}, units::VUnit, BBox, FrameRenderer, GridRenderer, MarginBox, WorldView};
pub struct UpdateManager {
    grid_renderer: GridRenderer,
    frame_renderer: FrameRenderer,

    ///index with frame handle to find the frame that owns a grid
    frame_to_grid_handle_map: Vec<Option<GridHandle>>,
}

pub enum UpdateMessage {
    Size(BBox),
}

impl  UpdateManager {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration, 
        world_view_layout: &wgpu::BindGroupLayout,
        world: &WorldView,
    ) -> Self {
        let mut ret = Self {
            grid_renderer: GridRenderer::new(device, config, world_view_layout),
            frame_renderer: FrameRenderer::new(device, config, world_view_layout),
            frame_to_grid_handle_map: vec![],
        };
        ret.frame_renderer.add(FrameData {
            data: BBox {x: 0.into(), y: 0.into(), w: world.w, h: world.h},
            margin: MarginBox::zeroed(),
            color: [255; 4],
            camera_index: 0,
            _pad1: 0
        });
        return ret;
    }
    pub fn update_world(&mut self, world: &WorldView) {
        self.update(0, &UpdateMessage::Size(BBox{x: VUnit(0),y: VUnit(0),w: world.w, h: world.h}));
    }
    pub fn update(&mut self, frame_handle: &FrameHandle, message: &UpdateMessage) {
        match message {
            UpdateMessage::Size(bounds) => {
                self.frame_renderer.update(frame_handle, *bounds);
                if let Some(Some(grid_handle)) = self.frame_to_grid_handle_map.get(frame_handle.index()) {
                    self.grid_renderer.update(grid_handle, bounds, &mut self.frame_renderer);
                }
            }
        }
    }
    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        self.frame_renderer.prepare(queue);
        self.grid_renderer.prepare(queue);
    }
    pub fn add_frame(&mut self, grid_handle: GridHandle) -> FrameHandle {
        self.frame_to_grid_handle_map.push(None);
        let fh = self.frame_renderer.add(FrameData {
            data: BBox::zeroed(),
            margin: MarginBox {top: VUnit(25), bottom: VUnit(25), left: VUnit(25), right: VUnit(25)},
            color: [255,255,255,25 ],
            camera_index: (self.frame_to_grid_handle_map.len() - 1) as u16,
            _pad1: 0,
        });
        self.grid_renderer.add_frame(grid_handle, fh);
        return fh;
    }
    pub fn add_grid(&mut self, 
        parent_frame: usize, 
        x_spacer: GridSpacer, 
        y_spacer: GridSpacer, 
        repeat_dir: Option<GridRepeatDir>,
    ) -> usize {
        let handle = self.grid_renderer.add(Grid::new(parent_frame, x_spacer, y_spacer, repeat_dir));
        self.frame_to_grid_handle_map[parent_frame] = Some(handle);
        return handle;
    }
    pub fn render<'a>(&'a self,render_pass: &mut wgpu::RenderPass<'a>) {
        self.frame_renderer.render(render_pass);
        self.grid_renderer.render(render_pass); //no op
    }
}
