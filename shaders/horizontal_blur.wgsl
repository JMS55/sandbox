struct Uniform {
    texture_size: vec2<f32>;
};

struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
};

[[group(0), binding(0)]]
var texture_sampler: sampler;
[[group(0), binding(1)]]
var input_texture: texture_2d<f32>;
[[group(0), binding(2)]]
var<uniform> global: Uniform;
var<private> texture_coordinates1: vec2<f32>;
var<private> color: vec4<f32>;

fn main1() {
    var i: i32 = -4;
    var coordinates: vec2<f32>;

    color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    loop {
        let e14: i32 = i;
        if (!((e14 <= 4))) {
            break;
        }
        {
            let e21: vec2<f32> = texture_coordinates1;
            coordinates = e21;
            let e24: vec2<f32> = coordinates;
            let e26: i32 = i;
            let e27: vec2<f32> = global.texture_size;
            coordinates.x = (e24.x + (f32(e26) / (e27.x - 1.0)));
            let e34: vec4<f32> = color;
            let e36: vec4<f32> = color;
            let e39: vec2<f32> = coordinates;
            let e40: vec4<f32> = textureSample(input_texture, texture_sampler, e39);
            let e42: vec3<f32> = (e36.xyz + e40.xyz);
            color.x = e42.x;
            color.y = e42.y;
            color.z = e42.z;
        }
        continuing {
            let e18: i32 = i;
            i = (e18 + 1);
        }
    }
    let e49: vec4<f32> = color;
    let e51: vec4<f32> = color;
    let e55: vec3<f32> = (e51.xyz / vec3<f32>(9.0));
    color.x = e55.x;
    color.y = e55.y;
    color.z = e55.z;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] texture_coordinates: vec2<f32>) -> FragmentOutput {
    texture_coordinates1 = texture_coordinates;
    main1();
    let e13: vec4<f32> = color;
    return FragmentOutput(e13);
}
