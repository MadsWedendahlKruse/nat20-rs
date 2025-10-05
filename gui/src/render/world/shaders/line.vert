// line.vert (420 core)
#version 420 core
layout(location=0) in vec3 a_pos;
layout(location=1) in vec3 a_col;

layout(std140, binding=0) uniform Frame {
    mat4 u_view_proj;
    vec4 u_light_dir; // unused here
};

uniform mat4 u_model; // optional; identity for world-space

out vec3 v_col;

void main() {
    gl_Position = u_view_proj * u_model * vec4(a_pos, 1.0);
    v_col = a_col;
}
