// Vertex shader

struct VertexInput {
    @builtin(instance_index) index: u32,
    @location(0) position: vec2<f32>,
    @location(1) vertex_xywh: vec4<i32>,
    @location(2) margin: vec4<i32>,
    @location(3) color: vec4<f32>,
    @location(4) camera_index: u32,
};

const VUNIT_PRECISION = 64; // 1 << 6

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct CameraArray {
    xywh: vec4<i32> 
};

@group(0) @binding(0) 
var<storage, read> camera_array: array<CameraArray>;


/// expects box to be xywh and margin [top, bottom, right, left]
fn calculate_margin(box: vec4<i32>, margin: vec4<i32>) -> vec4<i32> {
    var res = box - vec4<i32>(-margin.z, -margin.x, margin.z + margin.w, margin.x + margin.y);
    res = max(res, vec4<i32>(0));
    return res;
}

@vertex
fn vs_main(
    v: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    let xywh = calculate_margin(v.vertex_xywh, v.margin);
    let abs_pos = vec2<f32>(xywh.xy);
    let abs_dim = vec2<f32>(xywh.zw);
    let cam = vec4<f32>(camera_array[v.camera_index].xywh);
    let rel_pos: vec2<f32> = (abs_pos + cam.xy) / cam.zw;
    let rel_dim: vec2<f32> = abs_dim / cam.zw;
    out.clip_position = vec4<f32>( (((v.position / vec2<f32>(2.0, -2.0) + vec2<f32>(0.5, 0.5))  * rel_dim  + rel_pos) * vec2<f32>(2.0,-2.0) - vec2<f32>(1.0, -1.0)), 1.0,1.0);
    out.color = v.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

struct VertexIndexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: u32,
};

@vertex
fn vs_index_main(
    v: VertexInput
) -> VertexIndexOutput {
    var out: VertexIndexOutput;
    let xywh = calculate_margin(v.vertex_xywh, v.margin);
    let abs_pos = vec2<f32>(xywh.xy);
    let abs_dim = vec2<f32>(xywh.zw);
    let cam = vec4<f32>(camera_array[v.camera_index].xywh);
    let rel_pos: vec2<f32> = (abs_pos + cam.xy) / cam.zw;
    let rel_dim: vec2<f32> = abs_dim / cam.zw;
    out.clip_position = vec4<f32>( (((v.position / vec2<f32>(2.0, -2.0) + vec2<f32>(0.5, 0.5))  * rel_dim  + rel_pos) * vec2<f32>(2.0,-2.0) - vec2<f32>(1.0, -1.0)), 1.0,1.0);
    out.color = v.index;
    return out;
}

@fragment
fn fs_index_main(in: VertexIndexOutput) -> @location(0) u32 {
    return in.color;
}