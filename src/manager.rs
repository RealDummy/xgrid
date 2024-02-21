use std::{sync::mpsc, thread::JoinHandle};

use bytemuck::Zeroable;

use crate::{
    frame::{FrameData, FrameHandle},
    grid::{Grid, GridBuilder, GridHandle, XName, YName},
    handle::HandleLike,
    BBox, Borders, FrameRenderer, GridRenderer, MarginBox, WorldView,
};
pub struct UpdateManager {
    enquer: mpsc::Sender<UpdateMessage>,
    queue: mpsc::Receiver<UpdateMessage>,
    grid_renderer: GridRenderer,
    frame_renderer: FrameRenderer,

    ///index with frame handle to find the frame that owns a grid
    frame_to_grid_handle_map: Vec<Option<GridHandle>>,

    window_handle: FrameHandle,
}


pub type UpdateCycleId = u32;

pub enum UpdateMessage {
    FrameSize(UpdateCycleId,FrameHandle, BBox),
    Done(UpdateCycleId, mpsc::SyncSender<()>),
}

impl UpdateManager {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        world: &WorldView,
        send: mpsc::Sender<UpdateMessage>,
        recv: mpsc::Receiver<UpdateMessage>,
    ) -> Self {
        let mut frame_renderer = FrameRenderer::new(device, config);
        let window_handle = frame_renderer.add(FrameData {
            data: BBox {
                x: 0.into(),
                y: 0.into(),
                w: world.w,
                h: world.h,
            },
            margin: MarginBox::zeroed(),
            color: [255, 255, 255, 30],
            camera_index: 0,
        });
        Self {
            grid_renderer: GridRenderer::new(device, config),
            frame_renderer,
            frame_to_grid_handle_map: vec![None],
            window_handle,
            queue: recv,
            enquer: send,

        }
    }
    pub fn window(&self) -> FrameHandle {
        self.window_handle
    }
    pub fn update_world(&mut self, world: &WorldView) {
        self.update_frame(self.window(), *world);
    }
    pub fn update_all(&mut self) {
        loop {
            let mut done_sender = None;
            match self.queue.recv().unwrap() {
                UpdateMessage::FrameSize(id, handle, bounds) => {
                    self.update_frame(handle, bounds);
                }
                UpdateMessage::Done(id, done_sender) => {
                    
                }
            }
        }
    }
    pub fn update_frame(&mut self, frame_handle: FrameHandle, size: BBox) {
            self.frame_renderer.update(frame_handle, bounds);
            if let Some(Some(grid_handle)) =
                self.frame_to_grid_handle_map.get(frame_handle.index())
            {
                self.grid_renderer
                    .update(*grid_handle, bounds, &mut self.frame_renderer);
            }
        }
    }
    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        self.frame_renderer.prepare(queue);
        self.grid_renderer.prepare(queue);
    }
    pub fn add_frame(&mut self, grid_handle: GridHandle, x: XName, y: YName) -> FrameHandle {
        self.frame_to_grid_handle_map.push(None);
        let fh = self.frame_renderer.add(FrameData {
            data: BBox::zeroed(),
            margin: Borders {
                top: 10,
                bottom: 10,
                left: 10,
                right: 10,
            }
            .into(),
            color: [255, 255, 255, 25],
            camera_index: self.grid_renderer.get_parent_handle(grid_handle).index() as u32,
        });
        self.grid_renderer.add_frame(&mut self.frame_renderer, grid_handle, fh, x, y);
        return fh;
    }
    pub fn create_grid_in(&mut self, parent_frame: FrameHandle) -> GridBuilder {
        GridBuilder::new(parent_frame)
    }
    pub(crate) fn add_grid(&mut self, grid: Grid) -> GridHandle {
        self.grid_renderer.add(grid)
    }
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.frame_renderer.render(render_pass);
        self.grid_renderer.render(render_pass); //no op
    }
}

pub struct UpdateQueue {
    sender: mpsc::Sender<UpdateMessage>,
}

impl UpdateQueue {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        world: &WorldView,
    ) {
        let (send, recv) = mpsc::channel::<UpdateMessage>();
        std::thread::spawn(move || {
            let mut manager = UpdateManager::new(device, config, world, sender.clone(), recv);
            loop {
                manager.update_all();
                ok_sender.send(()).unwrap();
            }
            
        });
    }
}
