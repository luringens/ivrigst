#version 450

#define DSC_NONE 0
#define DSC_HUE 1
#define DSC_SATURATION 2
#define DSC_VALUE 3

layout(location = 0) out vec4 o_Target;

layout(binding = 0) uniform sampler2DShadow shadowtexture;
layout(binding = 1) uniform sampler2DShadow hatchingtexture;

uniform vec3 camera_position;
uniform vec3 light_vector;
uniform vec3 color;
uniform vec3 distance_shading_closest;
uniform vec3 distance_shading_furthest;
uniform float model_size;
uniform float distance_shading_power;
uniform uint distance_shading_channel;
uniform float distance_shading_constrict;
uniform float toon_factor;
uniform float shadow_intensity;
uniform float vertex_color_mix;
uniform uint hatching_frequency;
uniform float hatching_intensity;
uniform float hatching_far_plane;
uniform bool replace_shadows_with_hatching;

layout(location = 0) in vec3 normal_vector;
layout(location = 1) in vec3 toon_light_vector;
layout(location = 2) in vec3 position_vector;
layout(location = 3) in vec3 vertex_color;
layout(location = 4) in vec4 uv;
layout(location = 5) in vec4 hatchpos;

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

vec3 poissonDisk[16] = vec3[]( 
   vec3( -0.94201624, -0.39906216, 0.0 ), 
   vec3( 0.94558609, -0.76890725, 0.0 ), 
   vec3( -0.094184101, -0.92938870, 0.0 ), 
   vec3( 0.34495938, 0.29387760, 0.0 ), 
   vec3( -0.91588581, 0.45771432, 0.0 ), 
   vec3( -0.81544232, -0.87912464, 0.0 ), 
   vec3( -0.38277543, 0.27676845, 0.0 ), 
   vec3( 0.97484398, 0.75648379, 0.0 ), 
   vec3( 0.44323325, -0.97511554, 0.0 ), 
   vec3( 0.53742981, -0.47373420, 0.0 ), 
   vec3( -0.26496911, -0.41893023, 0.0 ), 
   vec3( 0.79197514, 0.19090188, 0.0 ), 
   vec3( -0.24188840, 0.99706507, 0.0 ), 
   vec3( -0.81409955, 0.91437590, 0.0 ), 
   vec3( 0.19984126, 0.78641367, 0.0 ), 
   vec3( 0.14383161, -0.1410079, 0.00 ) 
);

float random(vec3 seed, int i){
	vec4 seed4 = vec4(seed,i);
	float dot_product = dot(seed4, vec4(12.9898,78.233,45.164,94.673));
	return fract(sin(dot_product) * 43758.5453);
}

float ShadowCalculation(vec4 fragPosLightSpace)
{
    float cosTheta = clamp(dot(light_vector, vec3(1)), 0.0, 1.0);
    float bias = 0.005 * tan(acos(cosTheta));
    bias = clamp(bias, 0.0, 0.05);

    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    float currentDepth = projCoords.z;
    
    float shadow = 1.0;
    for (int i=0;i<16;i++){
        
        int index = int(16.0 * random(gl_FragCoord.xyy, i))%16;;
        float sample_depth = texture(shadowtexture, projCoords.xyz + poissonDisk[index] / 700.0 ).r;
        if (sample_depth < currentDepth - bias) {
            shadow-= shadow_intensity / 16.0;
        }
    }
    if (projCoords.z > 1.0) {
        shadow = 0.0;
    }
    return shadow;
}

float triangle(float x) {
    // Put on a -1 to 1 range
    x = float(int(floor(abs(x))) % hatching_frequency) / hatching_frequency * 2.0 - 1.0;

    // Triangle function
    return max(0, 1.0 - abs(x));
}

float hatchingCalculation()
{
    vec3 projCoords = vec3(0);
    projCoords.xy = hatchpos.xy / hatchpos.w * 0.5 + 0.5;
    projCoords.z = hatchpos.z / hatching_far_plane;
    projCoords.z -= 0.01;
    float sample_depth = texture(hatchingtexture, projCoords).r;

    if (sample_depth < 0.5) {
        return triangle(gl_FragCoord.x - gl_FragCoord.y);
    }

    return 1.0;
}

void main() {
    vec3 color = mix(color, vertex_color, vertex_color_mix);

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
    // float camera_dist = length(camera_position);
    // float near_plane = camera_dist - model_size / 2.0 * distance_shading_constrict;
    // float far_plane = camera_dist + model_size / 2.0 * distance_shading_constrict;
    float near_plane = length(camera_position - distance_shading_closest);
    float far_plane = length(camera_position - distance_shading_furthest);

    float power = distance_shading_power;

    // For the Hue channel, it does not make sense to restrict available colour space.
    if (distance_shading_channel == DSC_HUE) {
        power = 1.0;
    }

    // Calculate magnitude of shading.
    // float z = abs(gl_FragCoord.z / gl_FragCoord.w);
    float z = length(camera_position - position_vector);
    float d = 1.0 - smoothstep(near_plane, far_plane, z) * power;
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
    if (!replace_shadows_with_hatching) {
        float shadow = shadow_intensity < 0.005 ? 1.0 : ShadowCalculation(uv);
        color.z *= shadow;
    }

    color = hsv2rgb(color);

    // Hatching
    if (replace_shadows_with_hatching) {
        float hatching = hatchingCalculation();
        if (hatching < 1.0) 
        {
            vec3 shadow_color = color / vec3(3, 3, 1.5);
            hatching = (1.0 - hatching) * hatching_intensity;
            color = mix(color, shadow_color, hatching);
        }
    }

    o_Target = vec4(color, 1);
}
