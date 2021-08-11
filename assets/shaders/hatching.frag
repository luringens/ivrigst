#version 330 core

uniform uint steps;

void main(){
    float depth = gl_FragCoord.z / gl_FragCoord.w;
    gl_FragDepth = floor(depth * steps) / steps;
}
