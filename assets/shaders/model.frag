#version 450

#define DSC_NONE 0
#define DSC_HUE 1
#define DSC_SATURATION 2
#define DSC_VALUE 3

layout(location = 0) out vec4 o_Target;

uniform sampler2DShadow shadowtexture;

uniform vec3 camera_position;
uniform vec3 light_vector;
uniform vec3 color;
uniform float model_size;
uniform float distance_shading_power;
uniform uint distance_shading_channel;
uniform float distance_shading_constrict;
uniform float toon_factor;
uniform bool shadows;

layout(location = 0) in vec3 normal_vector;
layout(location = 1) in vec3 toon_light_vector;
layout(location = 2) in vec3 position_vector;
layout(location = 3) in vec4 uv;

// https://stackoverflow.com/a/17897228
// All components are in the range [0…1], including hue.
vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// All components are in the range [0…1], including hue.
vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec2 poissonDisk[4] = vec2[](
    vec2(-0.94201624,  -0.39906216),
    vec2( 0.94558609,  -0.76890725),
    vec2(-0.094184101, -0.92938870),
    vec2( 0.34495938,   0.29387760)
);

float ShadowCalculation(vec4 fragPosLightSpace)
{
    float cosTheta = clamp(dot(light_vector, vec3(1)), 0.0, 1.0);
    float bias = 0.005 * tan(acos(cosTheta));
    bias = clamp(bias, 0.0, 0.01);

    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    float closestDepth = texture(shadowtexture, projCoords.xyz).r;
    float currentDepth = projCoords.z;
    float shadow = 0.0;
    if (closestDepth < currentDepth - bias) {
        shadow = 1.0;
    }
    if (projCoords.z > 1.0) {
        shadow = 0.0;
    }
    return shadow;
}


void main() {
    vec3 color = color;
    float shadow = shadows ? ShadowCalculation(uv) : 0.0;

    vec3 toonShadingColor;
    {
        vec3 cl = color;    
        vec3 light = -normalize(toon_light_vector.xyz);    
        float vdn = light.z;
        cl *= round(vdn * 5) / 5;
        cl *= vdn;
        if (vdn < 0.3)
        {
            cl = vec3(0);
        }
        toonShadingColor = cl;
    }

    vec3 standardShadingColor;
    {
        float ambientReflection = 0.3;
        float ambientIntensity = 1;

        float diffuseReflection = 0.5;
        float diffuseIntensity = 1;

        float specularReflection = 0.9;
        float specularIntensity = 1;

        float shininess = 5;

        // Vector to camera
        vec3 v = normalize(camera_position - position_vector);

        // Vector to light source
        vec3 light_position = camera_position;
        vec3 lm = normalize(light_position - position_vector);

        // Reflected light vector
        vec3 np = 2 * normalize(dot(lm, normal_vector) * normal_vector);
        vec3 rm = normalize(np - lm);

        // Light intensity
        float ip = ambientReflection * ambientIntensity + (diffuseReflection * diffuseIntensity * dot(lm, normal_vector) + specularReflection * specularIntensity * pow(max(0, min(1, dot(rm, v))), shininess));

        standardShadingColor = ip * color.xyz;
    }
    color = mix(standardShadingColor, toonShadingColor, toon_factor);

    // Reduce Value of colour based on distance from camera.    
    float camera_dist = length(camera_position);
    float near_plane = camera_dist - model_size / 2.0 * distance_shading_constrict;
    float far_plane = camera_dist + model_size / 2.0 * distance_shading_constrict;

    float power = distance_shading_power;

    // For the Hue channel, it does not make sense to restrict available colour space.
    if (distance_shading_channel == DSC_HUE) {
        power = 1.0;
    }

    // Calculate magnitude of shading.
    float z = abs(gl_FragCoord.z / gl_FragCoord.w / 1);
    float d = 1.0 - min(smoothstep(near_plane, far_plane, z), power);
    color = rgb2hsv(color);

    // Perform shading on channel of choice.
    if (distance_shading_channel == DSC_HUE) {
        color.x = d;
    }
    else if (distance_shading_channel == DSC_SATURATION) {
        color.y *= d;
    }
    else if (distance_shading_channel == DSC_VALUE) {
        color.z *= d;
    }

    // Shadows
    color.z *= max(1.0 - shadow, 0.3);

    color = hsv2rgb(color);

    o_Target = vec4(color, 1);
}
