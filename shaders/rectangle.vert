#version 450

layout(location = 0) out vec2 texture_coordinates;

void main() {
    texture_coordinates.x = (gl_VertexIndex == 2) ? 2.0 : 0.0;
    texture_coordinates.y = (gl_VertexIndex == 1) ? 2.0 : 0.0;
    gl_Position = vec4(texture_coordinates * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 1.0, 1.0);
}
