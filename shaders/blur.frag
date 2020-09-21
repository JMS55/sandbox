#version 450

layout(set = 0, binding = 0) uniform sampler texture_sampler;
layout(set = 0, binding = 1) uniform texture2D input_texture;
layout(set = 0, binding = 2) uniform Uniform { vec2 texture_size; };
layout(location = 0) in vec2 texture_coordinates;
layout(location = 0) out vec4 color;

void main() {
    color = vec4(0.0, 0.0, 0.0, 1.0);
    for (int i = -4; i <= 4; i++) {
        vec2 coordinates = texture_coordinates;
        #ifdef HORIZONTAL
            coordinates.x += i / (texture_size.x - 1.0);
        #endif
        #ifdef VERTICAL
            coordinates.y += i / -(texture_size.y - 1.0);
        #endif
        color.rgb += texture(sampler2D(input_texture, texture_sampler), coordinates).rgb;
    }
    color.rgb /= 9.0;
}
