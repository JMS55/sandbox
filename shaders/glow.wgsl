struct VertexOutput {
    @location(0) texture_coordinates: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex
fn fullscreen_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var texture_coordinates: vec2<f32>;
    texture_coordinates.x = select(0.0, 2.0, (vertex_index == u32(2)));
    texture_coordinates.y = select(0.0, 2.0, (vertex_index == u32(1)));
    var position: vec4<f32> = vec4<f32>(((texture_coordinates * vec2<f32>(2.0, -(2.0))) + vec2<f32>(-(1.0), 1.0)), 1.0, 1.0);
    return VertexOutput(texture_coordinates, position);
}

struct Uniform {
    texture_size: vec2<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0)
var texture_sampler: sampler;
@group(0) @binding(1)
var input_texture: texture_2d<f32>;
@group(0) @binding(2)
var<uniform> global: Uniform;
var<private> color: vec4<f32>;

fn blur(horizontal: bool, texture_coordinates: vec2<f32>) {
    color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    for (var i: i32 = -4; i <= 4; i = i + 1) {
        var coordinates: vec2<f32> = texture_coordinates;
        if (horizontal) {
            coordinates.x = (coordinates.x + (f32(i) / (global.texture_size.x - 1.0)));
        } else {
            coordinates.y = (coordinates.y + (f32(i) / (global.texture_size.y - 1.0)));
        }
        let tex_sample: vec4<f32> = textureSample(input_texture, texture_sampler, coordinates);
        color = vec4<f32>(color.xyz + tex_sample.xyz, color.a);
    }
    color = vec4<f32>((color.xyz / vec3<f32>(9.0)).xyz, color.a);
}

@fragment
fn blur_horizontal_main(@location(0) texture_coordinates: vec2<f32>) -> FragmentOutput {
    blur(true, texture_coordinates);
    return FragmentOutput(color);
}

@fragment
fn blur_vertical_main(@location(0) texture_coordinates: vec2<f32>) -> FragmentOutput {
    blur(false, texture_coordinates);
    return FragmentOutput(color);
}

@fragment
fn copy_glowing_main(@location(0) texture_coordinates: vec2<f32>) -> FragmentOutput {
    color = textureSample(input_texture, texture_sampler, texture_coordinates);
    color = mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), color, vec4<f32>(step(color.a, 0.0)));
    return FragmentOutput(color);
}

@group(0) @binding(1)
var input_texture1: texture_2d<f32>;
@group(0) @binding(2)
var input_texture2: texture_2d<f32>;

@fragment
fn combine_main(@location(0) texture_coordinates: vec2<f32>) -> FragmentOutput {
    color = textureSample(input_texture1, texture_sampler, texture_coordinates);
    color = color + textureSample(input_texture2, texture_sampler, texture_coordinates);
    color.a = 1.0;
    return FragmentOutput(color);
}
