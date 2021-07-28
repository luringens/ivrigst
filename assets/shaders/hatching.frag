#version 330 core

const float STEPS = 5.0;

void main(){
    float depth = gl_FragCoord.z / gl_FragCoord.w;
    // gl_FragDepth = floor(depth * STEPS) / STEPS;
    gl_FragDepth = depth;
}
