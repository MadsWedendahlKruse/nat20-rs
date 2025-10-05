// shaders/basic.vert
#version 420 core
layout (location=0) in vec3 a_pos;
layout (location=1) in vec3 a_nrm;

layout(std140, binding=0) uniform Frame {
    mat4 u_view_proj;
    vec4 u_light_dir; // xyz used, w unused
};

uniform mat4 u_model;

out vec3 v_nrm;

void main() {
    gl_Position = u_view_proj * u_model * vec4(a_pos, 1.0);
    // If you might use non-uniform scale later, pass a normal matrix.
    v_nrm = mat3(u_model) * a_nrm;
}
