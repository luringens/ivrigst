#version 330 core

uniform uint steps;
uniform float far_plane;

void main(){
    // float depth = (gl_FragCoord.z / gl_FragCoord.w) / far_plane;
    // gl_FragDepth = floor(depth * steps) / steps;
    float depth = (gl_FragCoord.z / gl_FragCoord.w) / far_plane;
    gl_FragDepth = depth;
}
