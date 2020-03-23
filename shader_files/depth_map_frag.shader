#version 330 core

void main() {             
    // set by default but nice to be explicit
    gl_FragDepth = gl_FragCoord.z;
}  