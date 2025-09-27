// basic.frag
#version 420 core
in vec3 v_nrm;
layout(std140, binding=0) uniform Frame {
    mat4 u_view_proj;
    vec4 u_light_dir;
};
out vec4 FragColor;

void main() {
    float NdotL = max(dot(normalize(v_nrm), normalize(-u_light_dir.xyz)), 0.0);
    vec3 base = vec3(0.75, 0.8, 0.9);
    vec3 color = base * (0.25 + 0.75 * NdotL);
    FragColor = vec4(color, 1.0);
}
