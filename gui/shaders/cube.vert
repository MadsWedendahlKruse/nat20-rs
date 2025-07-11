#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

uniform mat4 u_mvp;
uniform mat4 u_model;

out vec3 frag_normal;

void main() {
    frag_normal = mat3(transpose(inverse(u_model))) * normal; // Transform normal properly
    gl_Position = u_mvp * vec4(position, 1.0);
}
