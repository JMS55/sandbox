struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
};

[[group(0), binding(0)]]
var texture_sampler: sampler;
[[group(0), binding(1)]]
var input_texture1_: texture_2d<f32>;
[[group(0), binding(2)]]
var input_texture2_: texture_2d<f32>;
var<private> texture_coordinates1: vec2<f32>;
var<private> color: vec4<f32>;

fn main1() {
    let e6: vec2<f32> = texture_coordinates1;
    let e7: vec4<f32> = textureSample(input_texture1_, texture_sampler, e6);
    color = e7;
    let e8: vec4<f32> = color;
    let e10: vec2<f32> = texture_coordinates1;
    let e11: vec4<f32> = textureSample(input_texture2_, texture_sampler, e10);
    color = (e8 + e11);
    color.w = 1.0;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] texture_coordinates: vec2<f32>) -> FragmentOutput {
    texture_coordinates1 = texture_coordinates;
    main1();
    let e13: vec4<f32> = color;
    return FragmentOutput(e13);
}
