#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};


layout(location = 0) out vec3 normalVector;
layout(location = 1) out vec3 lightVector;
layout(location = 2) out vec3 positionVector;

void main() {
    normalVector = normalize(Vertex_Normal);
    lightVector = vec3(ViewProj * Model * vec4(normalVector, 0.0));
    positionVector = Vertex_Position;
    
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
