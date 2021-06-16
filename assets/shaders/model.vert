#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;

uniform mat4 ProjectionMatrix;

layout(location = 0) out vec3 normalVector;
layout(location = 1) out vec3 lightVector;
layout(location = 2) out vec3 positionVector;

void main() {
    normalVector = normalize(Vertex_Normal);
    lightVector = vec3(ProjectionMatrix * vec4(normalVector, 0.0));
    positionVector = Vertex_Position;
    
    gl_Position = ProjectionMatrix * vec4(Vertex_Position, 1.0);
}
