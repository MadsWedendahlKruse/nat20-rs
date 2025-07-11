#version 330 core

in vec3 frag_normal;
out vec4 frag_color;

uniform vec3 u_light_dir;

void main() {
    vec3 norm = normalize(frag_normal);
    float diff = max(dot(norm, normalize(-u_light_dir)), 0.0);
    vec3 base = vec3(0.3, 0.6, 0.9);
    vec3 shaded = base * diff;
    frag_color = vec4(shaded, 1.0);
}
