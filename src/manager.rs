use std::{io::Seek, iter, sync::mpsc, thread::JoinHandle};

use bytemuck::{Pod, Zeroable};
use log::{debug, warn};
use wgpu::{util::DeviceExt, SurfaceError};
use winit::{dpi::LogicalSize, event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::{self, EventLoop}, keyboard::{Key, NamedKey}, window::{self, Window, WindowBuilder}};

use crate::{
    frame::{FrameData, FrameHandle, FrameRenderer}, grid::{GridBuilder, GridData, GridHandle, GridRenderer, XName, YName}, handle::{Handle, HandleLike}, units::{UserUnits, VUnit}
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1.],
    }, // A
    Vertex {
        position: [-1., -1.],
    }, // B
    Vertex {
        position: [1.0, 1.],
    }, // C
    Vertex {
        position: [1.0, -1.0],
    }, // D
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
    pub fn state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: Some(wgpu::IndexFormat::Uint16),
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),

            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        }
    }
}



pub type WorldView = BBox;

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct BBox {
    pub(crate) x: VUnit,
    pub(crate) y: VUnit,
    pub(crate) w: VUnit,
    pub(crate) h: VUnit,
}

pub struct Rect<T: Into<VUnit>> {
    x: T,
    y: T,
    w: T,
    h: T,
}
impl<T: Into<VUnit>> Into<BBox> for Rect<T> {
    fn into(self) -> BBox {
        let Self { x, y, w, h } = self;
        BBox {
            x: x.into(),
            y: y.into(),
            w: w.into(),
            h: h.into(),
        }
    }
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct MarginBox {
    pub top: VUnit,
    pub bottom: VUnit,
    pub left: VUnit,
    pub right: VUnit,
}

pub struct Borders<T: Into<VUnit>> {
    pub top: T,
    pub bottom: T,
    pub left: T,
    pub right: T,
}

impl<T: Into<VUnit>> Into<MarginBox> for Borders<T> {
    fn into(self) -> MarginBox {
        let Self {
            top,
            bottom,
            left,
            right,
        } = self;
        MarginBox {
            top: top.into(),
            bottom: bottom.into(),
            left: left.into(),
            right: right.into(),
        }
    }
}

pub type WindowHandle = Handle<()>;

pub struct UpdateManager<'a> {
    vertex_buffer: wgpu::Buffer,
    surface: wgpu::Surface<'a>,
    window: &'a winit::window::Window,
    size: winit::dpi::LogicalSize<u32>,
    index_render_target: wgpu::Texture,
    base_handle: FrameHandle,
    grid_renderer: GridRenderer,
    frame_renderer: FrameRenderer,
    frame_to_grid_handle_map: Vec<Option<GridHandle>>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
}


pub type UpdateCycleId = u32;

pub enum UpdateMessage {
    FrameSize(UpdateCycleId,FrameHandle, BBox),
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub(crate) frame: FrameHandle,
    pub(crate) window: WindowHandle,
}

impl Frame {
    fn new(frame: FrameHandle, window: WindowHandle) -> Self {
        Self {
            frame,
            window,
        }
    }
    fn window(&self) -> usize {
        self.window.index()
    }
}

#[derive(Clone, Copy)]
pub struct Grid {
    pub(crate) grid: GridHandle,
    pub(crate) window: WindowHandle,
}

impl Grid {
    fn new(grid: GridHandle, window: WindowHandle) -> Self {
        Self {
            grid,
            window,
        }
    }
    fn window(&self) -> usize {
        self.window.index()
    }
}

// async fn create_window<'a>(event_loop: &EventLoop<()>, inner_size: winit::dpi::LogicalSize<u32>) -> LiveWindow<'a> {
//         let window = winit::window::WindowBuilder::new()
//         .with_inner_size(inner_size)
//         .build(event_loop)
//         .unwrap();

//         let size = inner_size;

//         // The instance is a handle to our GPU
//         // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        
//         LiveWindow {
//             window,
//             surface,
//             size,
//             grid_renderer,
//             frame_renderer,
//             base_handle: window_handle,
//             index_render_target,
//             frame_to_grid_handle_map: vec![],
//             config,
//             device,
//             queue,
//         }
// }

impl<'a> UpdateManager<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = LogicalSize::<i32> {
            width: 400,
            height: 400,
        };
        let world_view = WorldView {
            x: 0.into(),
            y: 0.into(),
            w: VUnit::new(size.width),
            h: VUnit::new(size.height),
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: *surface_caps
                .present_modes
                .iter()
                .find(|&&e| e == wgpu::PresentMode::Immediate)
                .unwrap_or(&surface_caps.present_modes[0]),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let index_render_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("index render target texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });


        let mut frame_renderer = FrameRenderer::new(&device, &config);
        let grid_renderer = GridRenderer::new(&device, &config);
        let window_handle = frame_renderer.add(FrameData {
            data: world_view,
            margin: MarginBox::zeroed(),
            color: [255, 255, 255, 30],
            camera_index: 0,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
         Self {
            size: size.cast(),
            vertex_buffer,
            index_render_target,
            surface,
            window,
            grid_renderer,
            frame_to_grid_handle_map: vec![None],
            base_handle: window_handle,
            frame_renderer,
            config,
            device,
            queue,
        }
    }
    
    pub fn update_frame(&mut self, frame_handle: Frame, size: BBox) {
        self.frame_renderer.update(frame_handle.frame, &size);
        if let Some(Some(grid_handle)) =
            self.frame_to_grid_handle_map.get(frame_handle.frame.index())
        {
            debug!("grid update");
            self.grid_renderer
                .update(*grid_handle, &size, &mut self.frame_renderer);
        }
    }
    pub fn prepare(&mut self) {
        self.grid_renderer.prepare(&self.queue);
        self.frame_renderer.prepare(&self.queue);
    }
    pub fn add_frame(&mut self, grid_handle: Grid, x: XName, y: YName) -> FrameHandle {
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
            camera_index: self.grid_renderer.get_parent_handle(grid_handle.grid).index() as u32,
        });
        self.grid_renderer.add_frame(&mut self.frame_renderer, grid_handle.grid, fh, x, y);
        return fh;
    }
    pub fn create_grid_in(&mut self, parent_frame: Frame) -> GridBuilder {
        GridBuilder::new(parent_frame)
    }

    pub(crate) fn add_grid(&mut self, frame: Frame, grid: GridData) -> Grid {
        let grid_handle = self.grid_renderer.add(grid);
        self.frame_to_grid_handle_map[frame.frame.index()] = Some(grid_handle);
        return Grid::new(grid_handle, frame.window);
    }

    fn resize(&mut self, new_size: winit::dpi::LogicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size.cast();
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let cam = Rect {
                x: 0,
                y: 0,
                w: new_size.width as i32,
                h: new_size.height as i32,
            }.into();
            let frame = Frame {
                frame: self.base_handle,
                window: WindowHandle::new(0),
            };
            self.update_frame(frame, cam);
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
            self.prepare();
            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                self.frame_renderer.render(&mut render_pass);
            }
            {
                let index_view = self.index_render_target.create_view(&wgpu::TextureViewDescriptor::default());
                let mut index_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &index_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                index_render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                self.frame_renderer.render_index(&mut index_render_pass);
            }
            self.queue.submit(iter::once(encoder.finish()));
            output.present();
            Ok(())

    }
    fn input(&mut self, window_event: &WindowEvent) -> bool {
        false
    }

}



pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::builder().filter_level(log::LevelFilter::Error).filter_module("xgrid", log::LevelFilter::Trace).target(env_logger::Target::Stdout).init();
        }
    }


    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    // State::new uses async code, so we're going to wait for it to finish
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut updates = UpdateManager::new(&window).await;
    let mut g = updates.create_grid_in(Frame { frame: updates.base_handle, window: WindowHandle::new(0) });
    let [x1,x2,x3] = g.widths().add(UserUnits::Pixel(100)).add(UserUnits::Ratio(0.2)).add_expanding(UserUnits::Fraction(1)).assign();
    let [y1,y2,y3] = g.heights().add(UserUnits::Pixel(100)).add(UserUnits::Fraction(1)).add(UserUnits::Pixel(100)).assign();
    let g = g.build(&mut updates);
    updates.add_frame(g, x1, y1);
    updates.add_frame(g, x2, y3);
    updates.add_frame(g, x3, None);
    updates.add_frame(g, x3, None);
    updates.add_frame(g, x3, None);
    updates.add_frame(g, x3, y3);

    let exit_status = event_loop.run(move |event: Event<_>, target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == updates.window.id() => {
                if !updates.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        } => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            updates.resize(physical_size.to_logical(1.0));
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            ()
                            //self.scale(*scale_factor);
                        }
                        WindowEvent::RedrawRequested => {
                            match updates.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    updates.resize(updates.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                // We're ignoring timeouts
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
                            updates.window.request_redraw();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });
    if let Err(e) = exit_status {
        warn!("{e}")
    };
}
