#version 450

layout(set = 0, binding = 0) uniform sampler texture_sampler;
layout(set = 0, binding = 1) uniform texture2D input_texture;
layout(location = 0) in vec2 texture_coordinates;
layout(location = 0) out vec4 color;

void main() {
    color = texture(sampler2D(input_texture, texture_sampler), texture_coordinates);
    color = mix(vec4(0.0, 0.0, 0.0, 1.0), color, step(color.a, 0.0));
}
