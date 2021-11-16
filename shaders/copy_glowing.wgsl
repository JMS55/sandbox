struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
};

[[group(0), binding(0)]]
var texture_sampler: sampler;
[[group(0), binding(1)]]
var input_texture: texture_2d<f32>;
var<private> texture_coordinates1: vec2<f32>;
var<private> color: vec4<f32>;

fn main1() {
    let e5: vec2<f32> = texture_coordinates1;
    let e6: vec4<f32> = textureSample(input_texture, texture_sampler, e5);
    color = e6;
    let e13: vec4<f32> = color;
    let e16: vec4<f32> = color;
    let e25: vec4<f32> = color;
    let e26: vec4<f32> = color;
    let e29: vec4<f32> = color;
    color = mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), e25, vec4<f32>(step(e29.w, 0.0)));
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] texture_coordinates: vec2<f32>) -> FragmentOutput {
    texture_coordinates1 = texture_coordinates;
    main1();
    let e11: vec4<f32> = color;
    return FragmentOutput(e11);
}
