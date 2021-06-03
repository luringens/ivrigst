#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};


layout(location = 0) out vec3 n;
layout(location = 1) out vec3 l;

void main() {
    n = normalize(Vertex_Normal);
    l = vec3(ViewProj * Model * vec4(n,0));
    
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
