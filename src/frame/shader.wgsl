// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) vertex_xywh: vec4<i32>,
    @location(2) margin: vec4<i32>
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct WorldView {
    xywh: vec4<i32>
};

@group(0) @binding(0) // 1.
var<uniform> wv: WorldView;

@vertex
fn vs_main(
    v: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    let rel_pos: vec2<f32> =  vec2<f32>(1.0,-1.0) * ((vec2<f32>(v.vertex_xywh.xy) + vec2<f32>(wv.xywh.xy)) / vec2<f32>(wv.xywh.zw)*2.0 - vec2<f32>(1.0,1.0));
    let rel_dim: vec2<f32> = vec2<f32>(v.vertex_xywh.zw) / vec2<f32>(wv.xywh.zw);
    out.clip_position = vec4<f32>(v.position * rel_dim + rel_pos,1.0,1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0,1.0,1.0, 0.1);
}