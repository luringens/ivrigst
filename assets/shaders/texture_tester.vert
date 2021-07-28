#version 330 core

layout (location = 0) in vec2 vertex_position;
layout (location = 1) in vec2 texture_coordinate;

out vec2 uv;

void main() {
  gl_Position = vec4(vertex_position, 1.0, 1.0);
  uv = texture_coordinate;
}

