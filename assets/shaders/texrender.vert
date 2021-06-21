#version 330 core

layout (location = 0) in vec3 pos;

out vec2 frag_pos;

void main() {
    frag_pos = pos.xy;
    gl_Position = vec4(frag_pos, -1.0, 1.0);
}