use bytemuck::Zeroable;
use log::debug;


use crate::{frame::{FrameData, FrameHandle}, grid::{Grid, GridBuilder, GridHandle}, units::VUnit, BBox, Borders, FrameRenderer, GridRenderer, MarginBox, Rect, WorldView};
pub struct UpdateManager {
    grid_renderer: GridRenderer,
    frame_renderer: FrameRenderer,

    ///index with frame handle to find the frame that owns a grid
    frame_to_grid_handle_map: Vec<Option<GridHandle>>,

    window_handle: FrameHandle,
}

pub enum UpdateMessage {
    Size(BBox),
}

impl  UpdateManager {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration, 
        world: &WorldView,
    ) -> Self {
        let mut frame_renderer = FrameRenderer::new(device, config);
        let window_handle =frame_renderer.add(FrameData {
            data: BBox {x: 0.into(), y: 0.into(), w: world.w, h: world.h},
            margin: MarginBox::zeroed(),
            color: [255, 255, 255, 30],
            camera_index: 0,
        });
        Self {
            grid_renderer: GridRenderer::new(device, config),
            frame_renderer,
            frame_to_grid_handle_map: vec![None],
            window_handle,
        }
    }
    pub fn window(&self) -> FrameHandle {
        self.window_handle
    }
    pub fn update_world(&mut self, world: &WorldView) {
        self.update(&self.window(), &UpdateMessage::Size(*world));
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
            margin: Borders {top: 25, bottom: 25, left: 25, right: 25}.into(),
            color: [255,255,255,25 ],
            camera_index: (self.frame_to_grid_handle_map.len() - 1) as u32,
        });
        self.grid_renderer.add_frame(grid_handle, fh);
        return fh;
    }
    pub fn create_grid_in(&mut self, 
        parent_frame: FrameHandle, 
    ) -> GridBuilder {
        GridBuilder::new(parent_frame)
    }
    pub fn add_grid(&mut self, grid: Grid) -> GridHandle {
        let parent = grid.parent_frame_handle.index();
        let handle = self.grid_renderer.add(grid);
        self.frame_to_grid_handle_map[parent] = Some(handle);
        return handle;
    }
    pub fn render<'a>(&'a self,render_pass: &mut wgpu::RenderPass<'a>) {
        self.frame_renderer.render(render_pass);
        self.grid_renderer.render(render_pass); //no op
    }
}
