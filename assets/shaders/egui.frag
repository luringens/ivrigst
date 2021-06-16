#version 330 core

uniform sampler2D texture_sampler;
in vec4 vertex_color_rgba;
in vec2 vertex_texture_coordinate;

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

void main() {
  // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
  vec4 texture_rgba = texture2D(texture_sampler, vertex_texture_coordinate);

  gl_FragColor = srgba_from_linear(vertex_color_rgba * texture_rgba) / 255.0;
}
