// basic.frag
#version 420 core
in vec3 v_nrm;

layout(std140, binding=0) uniform Frame {
    mat4 u_view_proj;
    vec4 u_light_dir;
};

uniform vec4 u_color;     // per-draw color
uniform int  u_mode;      // 0 = lit, 1 = flat color (for wireframe)

out vec4 FragColor;

void main() {
    if (u_mode == 1) {                // FLAT
        FragColor = u_color;
        return;
    }
    // LIT
    float NdotL = max(dot(normalize(v_nrm), normalize(-u_light_dir.xyz)), 0.0);
    float lit = 0.25 + 0.75 * NdotL;
    FragColor = vec4(u_color.rgb * lit, u_color.a);
}
