use std::iter;

use bytemuck::{bytes_of, Pod, Zeroable};
use frame::FrameRenderer;
use grid::{Grid, GridRenderer, SpacerUnit};
use log::{info, warn};
use manager::UpdateManager;
use units::{UserUnits, VUnit};
use wgpu::{util::DeviceExt, BufferSlice, PresentMode};
use winit::{
    dpi::PhysicalSize, event::*, event_loop::{ControlFlow, EventLoop}, keyboard::{Key, NamedKey, PhysicalKey, SmolStr}, window::{Window, WindowBuilder}
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod units;
mod grid;
mod frame;
mod manager;
mod handle;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
    fn state() -> wgpu::PrimitiveState {
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

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1.],
    }, // A
    Vertex {
        position: [-1., -1.],
    }, // B
    Vertex {
        position: [1.0 , 1.],
    }, // C
    Vertex {
        position: [1.0, -1.0],
    }, // D
];

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct WorldView {
    w: VUnit,
    h: VUnit,
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct BBox {
    x: VUnit,
    y: VUnit,
    w: VUnit,
    h: VUnit,
}

pub struct Rect<T: Into<VUnit>> {
    x: T,
    y: T,
    w: T,
    h: T,
}
impl<T: Into<VUnit>> Into<BBox> for Rect<T> {
    fn into(self) -> BBox {
        let Self {
            x,y,w,h
        } = self;
        BBox {
            x: x.into(),
            y: y.into(),
            w: w.into(),
            h: h.into(),
        }
    }
}

#[derive(Pod, Zeroable, Clone, Copy)]
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
            top,bottom,left,right
        } = self;
        MarginBox {
            top: top.into(),
            bottom: bottom.into(),
            left: left.into(),
            right: right.into(),
        }
    }
}

struct State<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    vertex_buffer: wgpu::Buffer,
    surface: wgpu::Surface<'a>,
    window: &'a Window,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<f64>,
    update_manager: UpdateManager,
}


impl<'window> State<'window> {
    async fn new(window: &'window Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
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
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: *surface_caps.present_modes.iter()
                .find(|&&e| {
                    e == PresentMode::Immediate
                }).unwrap_or(&surface_caps.present_modes[0]),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        surface.configure(&device, &config);

        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let world_view = WorldView {
            w: VUnit::new(size.width as i32),
            h: VUnit::new(size.height as i32),
        };
        let world_view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("world view"),
            contents: bytes_of(&world_view),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let world_view_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("World view bg layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                }
            ],
        });


        let mut update_manager = UpdateManager::new(&device, &config, &world_view_bind_group_layout, &world_view);
        
        let mut builder = update_manager.create_grid_in(update_manager.window());
        let [x1,x2] = builder.widths()
                .add(UserUnits::Pixel(100))
                .add(UserUnits::Ratio(0.3))
                .assign();
            
        let [y1, y2] = builder.heights()
            .add(UserUnits::Pixel(200))
            .add(UserUnits::Fraction(1))
            .assign();

        let g = builder.build(&mut update_manager);

        for i in 0..12 {
            update_manager.add_frame(g);
  
        }

        Self {
            surface,
            device,
            queue,
            config,
            size: size.cast(),
            vertex_buffer,
            window,
            update_manager,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size.cast();
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.queue.write_buffer(&self.world_view_buffer, 0, bytes_of(&self.world_view));

            self.update_manager.update_world(&self.world_view);
        }
    }

    pub fn scale(&mut self, new_scale: f64) {
        info!("new scale is {new_scale}");
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        ()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        self.update_manager.prepare(&self.queue);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear (wgpu::Color {
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
            render_pass.set_bind_group(0, &self.world_view_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            self.update_manager.render(&mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::builder().filter_level(log::LevelFilter::Error).filter_module("xgrid", log::LevelFilter::Trace).target(env_logger::Target::Stdout).init();
        }
    }

    let event_loop = EventLoop::new().expect("event loop failed to new");
    let window = WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {width: 400, height: 400}).build(&event_loop).unwrap();

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
    let mut state = State::new(&window).await;

    let exit_status = event_loop.run(move |event: Event<_>, target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
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
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.scale(*scale_factor);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size.cast())
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                // We're ignoring timeouts
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
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
