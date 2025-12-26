#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 FragColor;
uniform sampler2D u_texture;
uniform float u_base_color;

void main() {
    float gray = texture(u_texture, v_tc).r;
    float v = u_base_color + gray * (1.0 - u_base_color);
    if (v < 0.5) {
        float t = v * 2.0;
        FragColor = vec4(0.0, 0.0, t, 1.0);
    } else {
        float t = (v - 0.5) * 2.0;
        FragColor = vec4(t, 0.0, 1.0 - t, 1.0);
    }
}