#version 330 core

uniform uint steps;
uniform float far_plane;

void main(){
    float depth = (gl_FragCoord.z / gl_FragCoord.w);
    gl_FragDepth = floor(depth * steps) / steps;
}
