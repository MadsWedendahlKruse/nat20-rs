#version 420 core
in vec3 v_col;
out vec4 FragColor;
void main() {
    FragColor = vec4(v_col, 1.0);
}
