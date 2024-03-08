use std::{
    iter,
    sync::{mpsc, Arc, Barrier},
    thread,
};

use bytemuck::{Pod, Zeroable};
use log::warn;
use wgpu::{util::DeviceExt, SurfaceError};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopProxy, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

use crate::{
    component::{self, ComponentBuilder, State},
    events::MouseEvent,
    frame::{FrameHandle, FrameRenderer},
    grid::GridRenderer,
    handle::HandleLike,
    render_actor::{FrameMessage, UpdateMessage},
    units::VUnit,
    update_queue::{
        self,
        back::QualifiedUpdateMsg,
        front::{self},
    },
    ButtonState, Component, MouseButton,
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
pub struct RenderManager<'a> {
    vertex_buffer: wgpu::Buffer,
    surface: wgpu::Surface<'a>,
    window: &'a winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
    index_render_target: wgpu::Texture,
    base_handle: FrameHandle,
    msg_send: mpsc::Sender<UpdateMessage>,
    msg_recv: mpsc::Receiver<UpdateMessage>,
    grid_to_frame_map: Vec<FrameHandle>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    frame_renderer: FrameRenderer,
    grid_renderer: GridRenderer,
    proxy: EventLoopProxy<()>,
}

impl<'a> RenderManager<'a> {
    pub async fn new(
        window: &'a Window,
        send: mpsc::Sender<UpdateMessage>,
        recv: mpsc::Receiver<UpdateMessage>,
        proxy: winit::event_loop::EventLoopProxy<()>,
    ) -> (update_queue::front::UpdateQueue, Self) {
        let size = LogicalSize::<i32> {
            width: 400,
            height: 400,
        }
        .to_physical(window.scale_factor());
        let _world_view = WorldView {
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
                .find(|&&e| e == wgpu::PresentMode::AutoVsync)
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let window_handle = FrameHandle::new(0);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let update_queue = front::UpdateQueue::new(&send);
        (
            update_queue,
            Self {
                frame_renderer: FrameRenderer::new(&device, &config),
                grid_renderer: GridRenderer::new(&device, &config),
                size: size.cast(),
                vertex_buffer,
                index_render_target,
                surface,
                window,
                msg_recv: recv,
                grid_to_frame_map: vec![],
                base_handle: window_handle,
                config,
                device,
                queue,
                msg_send: send,
                proxy,
            },
        )
    }
    pub fn window(&self) -> FrameHandle {
        self.base_handle
    }

    pub fn render(&'a self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
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
            let index_view = self
                .index_render_target
                .create_view(&wgpu::TextureViewDescriptor::default());
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
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }
    fn prepare(&mut self) {
        self.grid_renderer
            .prepare(&mut self.frame_renderer, &self.queue);
        self.frame_renderer.prepare(&self.queue);
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    pub fn run_forever(mut self, _barrier: Arc<Barrier>) {
        let mut count = 0;
        loop {
            count += 1;
            let msg = self.msg_recv.recv().expect("update message recv err");
            if let UpdateMessage::Exit = msg {}
            match msg {
                UpdateMessage::Draw => {
                    let rr = self.render();
                    let Err(e) = rr else {
                        self.window.request_redraw();
                        continue;
                    };
                    match e {
                        SurfaceError::Lost => self.resize(self.size),
                        _ => warn!("{e}"),
                    }
                }
                UpdateMessage::Prepare => self.prepare(),
                UpdateMessage::ModifyFrame(h, f) => {
                    let FrameMessage { size, color, .. } = f;
                    if let Some(size) = size {
                        self.frame_renderer.update(h.index(), &size);
                        if h.index() == 0 {
                            self.resize(PhysicalSize {
                                width: size.w.pix() as u32,
                                height: size.h.pix() as u32,
                            })
                        }
                    }
                    if let Some(color) = color {
                        self.frame_renderer.update_color(h.index(), color);
                    }
                }
                UpdateMessage::NewFrame(grid, x, y, f, frame_handle) => {
                    let FrameMessage {
                        size,
                        color,
                        margin,
                    } = f;
                    let size = size.unwrap_or(BBox::zeroed());
                    let color = color.unwrap_or([0; 4]);
                    let margin = margin.unwrap_or(MarginBox::zeroed());
                    self.frame_renderer.add(crate::FrameData {
                        data: size,
                        margin: margin,
                        color: color,
                        camera_index: self.grid_to_frame_map[grid.index()].index() as u32,
                    });
                    self.grid_renderer
                        .add_frame(&mut self.frame_renderer, grid, frame_handle, x, y)
                }
                UpdateMessage::NewFloatingFrame(f) => {
                    let FrameMessage {
                        size,
                        color,
                        margin,
                    } = f;
                    let size = size.unwrap_or(BBox::zeroed());
                    let color = color.unwrap_or([0; 4]);
                    let margin = margin.unwrap_or(MarginBox::zeroed());
                    self.frame_renderer.add(crate::FrameData {
                        data: size,
                        margin: margin,
                        color: color,
                        camera_index: 0,
                    });
                }
                UpdateMessage::ModifyGrid(_grid, _g) => (),
                UpdateMessage::NewGrid(_grid_index, grid_builder) => {
                    self.grid_to_frame_map.push(grid_builder.parent());
                    self.grid_renderer.add(grid_builder.build());
                }
                UpdateMessage::Exit => {
                    self.proxy.send_event(()).unwrap();
                    break;
                }
            }
        }
    }
}

pub fn run<App: State<Param = ()>>() {
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

    let event_loop = EventLoop::new().unwrap();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let (send, recv) = mpsc::channel();
    let barrier = Arc::new(Barrier::new(2));

    thread::scope(|s| {
        let (queue, renderer) =
            pollster::block_on(RenderManager::new(&window, send.clone(), recv, proxy));
        let mut builder = ComponentBuilder::new(send.clone(), queue.clone());
        let barrier_ref = &barrier;
        s.spawn(move || {
            renderer.run_forever(Arc::clone(barrier_ref));
        });
        let _app: Component<App> = builder.send_app(
            Rect {
                x: 0,
                y: 0,
                w: 400,
                h: 400,
            }
            .into(),
        );
        let exit_status =
            event_loop.run(move |event: Event<_>, target: &EventLoopWindowTarget<_>| {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id: _,
                    } => {
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
                            } => {
                                send.send(UpdateMessage::Exit).unwrap();
                            }
                            WindowEvent::Resized(physical_size) => queue.send(QualifiedUpdateMsg {
                                msg: crate::UpdateMsg::Frame(FrameMessage {
                                    size: Some(
                                        Rect {
                                            x: 0,
                                            y: 0,
                                            w: physical_size.width as i32,
                                            h: physical_size.height as i32,
                                        }
                                        .into(),
                                    ),
                                    color: None,
                                    margin: None,
                                }),
                                dst: component::ComponentType::Floating(FrameHandle::new(0)),
                            }),
                            WindowEvent::ScaleFactorChanged { .. } => {
                                ()
                                //self.scale(*scale_factor);
                            }
                            WindowEvent::RedrawRequested => {
                                send.send(UpdateMessage::Prepare).unwrap();

                                send.send(UpdateMessage::Draw).unwrap();
                            }
                            WindowEvent::MouseInput { state, .. } => {
                                builder.emit_mouse(MouseEvent::Click(MouseButton::Left(
                                    match state {
                                        ElementState::Pressed => ButtonState::Pressed,
                                        ElementState::Released => ButtonState::Released,
                                    },
                                )));
                            }
                            _ => {}
                        }
                    }
                    Event::UserEvent(()) => {
                        target.exit();
                    }
                    _ => {}
                }
            });
        if let Err(e) = exit_status {
            warn!("{e}")
        };
    });
}
