#version 330

precision mediump float;

// Input vertex data, different for all executions of this shader.
layout(location = 0) in vec2 vertexPosition;
layout(location = 1) in vec2 vertexUV;
layout(location = 2) in vec4 vertexColor;

attribute vec2 u_screen_size;

// Output data ; will be interpolated for each fragment.
out vec2 v_tc;
out vec4 v_rgba;

// 0-255 sRGB  from  0-1 linear
vec3 srgb_from_linear(vec3 rgb) {
  bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
  vec3 lower = rgb * vec3(3294.6);
  vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
  return mix(higher, lower, vec3(cutoff));
}

vec4 srgba_from_linear(vec4 rgba) {
  return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

void main(){
    gl_Position = vec4(
        2.0 * vertexPosition.x / u_screen_size.x - 1.0,
        1.0 - 2.0 * vertexPosition.y / u_screen_size.y,
        0.0,
        1.0
    );
    // egui encodes vertex colors in gamma spaces, so we must decode the colors here:
    v_rgba = linear_from_srgba(vertexColor);
    v_tc = vertexUV;
}