#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 vertex_color;

uniform mat4 projection_matrix;
uniform float hatching_depth;

void main() {
    vec3 normal = normalize(vertex_normal);
    vec3 position = vertex_position + normal * hatching_depth;
    
    gl_Position = projection_matrix * vec4(position, 1.0);
}
