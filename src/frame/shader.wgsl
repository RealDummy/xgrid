// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) vertex_xywh: vec4<i32>,
    @location(2) margin: vec4<i32>,
    @location(3) color: vec4<f32>,
    @location(4) camera_index: u32,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct WorldView {
    wh: vec2<i32>
}

struct CameraArray {
    xywh: vec4<f32> 
};

@group(1) @binding(0) 
var<storage, read> camera_array: array<CameraArray>;


@group(0) @binding(0) // 1.
var<uniform> wv: WorldView;

@vertex
fn vs_main(
    v: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    let xywh = v.vertex_xywh - vec4<i32>(-v.margin.z, -v.margin.x, v.margin.z + v.margin.w, v.margin.x + v.margin.y);
    let abs_pos = vec2<f32>(xywh.xy);
    let abs_dim = vec2<f32>(xywh.zw);
    let cam = vec4<f32>(camera_array[v.camera_index].xywh);
    let rel_pos: vec2<f32> = (abs_pos + cam.xy) / cam.zw;
    let rel_dim: vec2<f32> = abs_dim / vec2<f32>(cam.zw);
    out.clip_position = vec4<f32>( (((v.position / vec2<f32>(2.0, -2.0) + vec2<f32>(0.5, 0.5)) * rel_dim + rel_pos) * vec2<f32>(2.0,-2.0) - vec2<f32>(1.0, -1.0)), 1.0,1.0);
    out.color = v.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}