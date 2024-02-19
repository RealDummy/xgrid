use std::{mem::size_of, vec::Vec};

use bytemuck::{Pod, Zeroable};
use log::debug;
use wgpu::{include_wgsl, BufferUsages, Device, MultisampleState, RenderPass, RenderPipeline, RenderPipelineDescriptor, SurfaceConfiguration};

use crate::{handle::Handle, BBox, Vertex};

use super::FrameData;

pub struct FrameRenderer {
    data: Vec<FrameData>,
    pipeline: RenderPipeline,
    frame_buffer_handle: wgpu::Buffer,
    changed: Option<usize>,
    camera_bg_handle: wgpu::BindGroup,
    camera_buffer_handle: wgpu::Buffer,
    camera_data: Vec<Camera>,
}
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct Camera {
    bbox: BBox,
}

pub type FrameHandle = Handle<FrameData>;

impl FrameRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let shader = include_wgsl!("shader.wgsl");
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("frame shader"),
            source: shader.source
        });
        let camera_buffer_handle = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera buffer"),
            size: (size_of::<Camera>() * 1000) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera buffer layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                }
            ],
        });
        let camera_bg_handle = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bg"),
            layout: & camera_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer_handle.as_entire_binding(),
                }
            ]
        });
        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("frame pipeline layout"),
            bind_group_layouts: &[
                &camera_bg_layout,
            ],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);

        let color_targets = [Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("frame pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    FrameData::desc(),
                ]
            },
            primitive: Vertex::state(),
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &color_targets,
            }),
            multiview: None,
        };
        let pipeline = device.create_render_pipeline(&pipeline_descriptor);

        let buffer_handle = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame instance buffer"),
            size: FrameData::BUFFER_INIT_BYTE_COUNT,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            frame_buffer_handle: buffer_handle,
            data: vec![],
            changed: None,
            camera_buffer_handle,
            camera_data: vec![],
            camera_bg_handle,
        }
    }
    pub fn prepare(&mut self, queue: &wgpu::Queue){
                queue.write_buffer(&self.frame_buffer_handle, 0, bytemuck::cast_slice(&self.data[..]));
                queue.write_buffer(&self.camera_buffer_handle, 0, bytemuck::cast_slice(&self.camera_data[..]));

        self.changed = None;
        
    }
    pub fn render<'rp>(&'rp self, render_pass: &mut RenderPass<'rp> ) {
        //debug!("FRAME RENDER: {} frames", self.camera_data.len());
        //debug!("CAMERA: {:?}", self.camera_data[0].bbox);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(1, self.frame_buffer_handle.slice(..));
        render_pass.set_bind_group(0, &self.camera_bg_handle, &[]);
        render_pass.draw(0..4 as u32, 0..self.data.len() as u32);

    }
    pub fn add(&mut self, mut frame: FrameData) -> FrameHandle {
        self.camera_data.push(Camera { bbox: frame.data });
        frame.camera_index = (self.camera_data.len() - 1) as u32;
        self.data.push(frame);
        self.changed = Some(self.data.len() - 1);
        return FrameHandle::new(self.data.len() - 1);
    }
    pub fn update(&mut self, handle: &FrameHandle, bounds: BBox) {
        let frame = &mut self.data[handle.index()];
        frame.data = bounds;
        self.camera_data[frame.camera_index as usize].bbox = bounds;
        self.changed = match self.changed {
            None => Some(handle.index()),
            Some(u)=> Some(usize::max(u, handle.index())),
        }
    }
    pub fn get<'a>(&'a self, handle: &FrameHandle) -> &'a FrameData {
        &self.data[handle.index()]
    }
}