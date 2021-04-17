#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;

layout(location=0) out vec3 v_color;

// camera
layout(set=0, binding=0) 
uniform Uniforms {
    vec4 u_view_position; // unused
    mat4 u_view_proj;
};

void main() {
    // color
    v_color = a_color;

    // camera position
    gl_Position = u_view_proj * vec4(a_position, 1.0);
}