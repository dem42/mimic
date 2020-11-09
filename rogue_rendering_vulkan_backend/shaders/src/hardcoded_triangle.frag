#version 450
#extension GL_ARB_separate_shader_objects : enable

// link to vert output by means of location 
// -> variable name doesn't have to be the same
layout(location = 0) in vec3 fragColor;

// specify the index of the framebuffer (in this case 0)
layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);    
}