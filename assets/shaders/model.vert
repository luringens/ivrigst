#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;

uniform mat4 projection_matrix;

layout(location = 0) out vec3 normal_vector;
layout(location = 1) out vec3 light_vector;
layout(location = 2) out vec3 position_vector;

void main() {
    normal_vector = normalize(vertex_normal);
    light_vector = vec3(projection_matrix * vec4(normal_vector, 0.0));
    position_vector = vertex_position;
    
    gl_Position = projection_matrix * vec4(vertex_position, 1.0);
}
