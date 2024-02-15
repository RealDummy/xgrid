use std::vec::Vec;

use bytemuck::bytes_of;
use wgpu::{include_wgsl, Device, MultisampleState, RenderPass, RenderPipeline, RenderPipelineDescriptor, SurfaceConfiguration};

use crate::{Vertex, WorldView};

use super::FrameData;

use crate::units::VUnit;

pub struct FrameRenderer {
    data: Vec<FrameData>,
    pipeline: RenderPipeline,
    buffer_handle: wgpu::Buffer,
}

impl FrameRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration, world_view_layout: &wgpu::BindGroupLayout) -> Self {
        let shader = include_wgsl!("shader.wgsl");
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("frame shader"),
            source: shader.source
        });
        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("frame pipeline layout"),
            bind_group_layouts: &[
                world_view_layout,
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
            buffer_handle,
            data: vec![
                FrameData {
                    data: [VUnit(100); 4],
                    margin: [VUnit(0); 4]
                }
            ],
        }
    }
    pub fn prepare(&mut self, queue: &wgpu::Queue){
        queue.write_buffer(&self.buffer_handle, 0, bytemuck::cast_slice(&self.data.as_slice()))
    }
    pub fn render<'rp>(&'rp self, render_pass: &mut RenderPass<'rp> ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(1, self.buffer_handle.slice(..));
        render_pass.draw(0..4 as u32, 0..self.data.len() as u32)

    }
}