#version 450

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform MyMaterial_vectors {
    vec3 camera_position;
    vec3 color;
    vec2 distance_shading;
};
layout(set = 3, binding = 0) uniform MyMaterial_floats {
    float model_size;
};

layout(location = 0) in vec3 n;
layout(location = 1) in vec3 l;

void main() {
    float z = abs(gl_FragCoord.z / gl_FragCoord.w / 1);
    float d = 1.0 - min(smoothstep(distance_shading.x, distance_shading.y, z), 0.8);
    
    vec3 cl = color * d;
    vec3 light = -normalize(l.xyz);
    
    float vdn = light.z;
    cl = round(vdn * 5) / 5 * cl;
    o_Target = vec4(cl*vdn,1);
    if (vdn < 0.3)
    {
        o_Target = vec4(vec3(0),1);
    }
}
