struct CameraTransform {
    scale: vec2<f32>,
    offset: vec2<f32>,
}

struct Instance {
    @location(1) scale: vec2<f32>,
    @location(2) offset: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraTransform;

@vertex
fn vs_main(
    in: VertexInput,
    instance: Instance,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.color;
    out.clip_position = vec4<f32>(((in.position - instance.offset) * instance.scale - camera.offset) * camera.scale, 1.0, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
