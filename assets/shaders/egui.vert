#version 330 core

uniform vec2 screen_size;
layout (location = 0) in vec2 vertex_position;
layout (location = 1) in vec2 texture_coordinate;
layout (location = 2) in vec4 color_srgba; // 0-255 sRGB
out vec4 vertex_color_rgba;
out vec2 vertex_texture_coordinate;

// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, cutoff);
}

vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {
  // egui encodes vertex colors in gamma spaces, so we must decode the colors here:
  vertex_color_rgba = linear_from_srgba(color_srgba);
  vertex_texture_coordinate = texture_coordinate;

  gl_Position = vec4(
    2.0 * vertex_position.x / screen_size.x - 1.0,
    1.0 - 2.0 * vertex_position.y / screen_size.y,
    0.0,
    1.0
  );
}

