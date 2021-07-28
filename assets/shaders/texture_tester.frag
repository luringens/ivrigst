#version 330 core

uniform sampler2D texture_sampler;
in vec2 uv;

void main() {
    gl_FragColor = vec4(texture2D(texture_sampler, uv));
}
