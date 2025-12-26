#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 FragColor;
uniform sampler2D u_texture;
uniform float u_base_color;

void main() {
    float gray = texture(u_texture, v_tc).r;
    float v = u_base_color + gray * (1.0 - u_base_color);
    FragColor = vec4(v, v, v, 1.0);
}

