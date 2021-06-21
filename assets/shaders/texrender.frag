#version 330 core

out vec4 FragColor;
in vec2 frag_pos;
uniform sampler2D shadowtexture;

void main() {
    vec2 uv = (frag_pos + vec2(1.0)) / 2.0;
    vec4 color = texture(shadowtexture, uv);
    FragColor = vec4(vec3(color.r), 1.0);
}