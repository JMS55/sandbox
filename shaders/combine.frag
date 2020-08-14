#version 450

layout(set = 0, binding = 0) uniform sampler texture_sampler;
layout(set = 0, binding = 1) uniform texture2D input_texture1;
layout(set = 0, binding = 2) uniform texture2D input_texture2;
layout(location = 0) in vec2 texture_coordinates;
layout(location = 0) out vec4 color;

void main() {
    color = texture(sampler2D(input_texture1, texture_sampler), texture_coordinates);
    color += texture(sampler2D(input_texture2, texture_sampler), texture_coordinates);
    color.a = 1.0;
}
