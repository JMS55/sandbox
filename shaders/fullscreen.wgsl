struct VertexOutput {
    [[location(0)]] texture_coordinates: vec2<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> texture_coordinates: vec2<f32>;
var<private> gl_VertexIndex: u32;
var<private> gl_Position: vec4<f32>;

fn main1() {
    let e3: u32 = gl_VertexIndex;
    texture_coordinates.x = select(0.0, 2.0, (e3 == u32(2)));
    let e11: u32 = gl_VertexIndex;
    texture_coordinates.y = select(0.0, 2.0, (e11 == u32(1)));
    let e19: vec2<f32> = texture_coordinates;
    gl_Position = vec4<f32>(((e19 * vec2<f32>(2.0, -(2.0))) + vec2<f32>(-(1.0), 1.0)), 1.0, 1.0);
    return;
}

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] param: u32) -> VertexOutput {
    gl_VertexIndex = param;
    main1();
    let e5: vec2<f32> = texture_coordinates;
    let e7: vec4<f32> = gl_Position;
    return VertexOutput(e5, e7);
}
