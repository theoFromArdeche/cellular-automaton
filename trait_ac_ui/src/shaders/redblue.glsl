#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 FragColor;
uniform sampler2D u_texture;

void main() {
    float gray = texture(u_texture, v_tc).r;
    if (gray < 0.5) {
        float t = gray * 2.0;
        FragColor = vec4(0.0, 0.0, t, 1.0);
    } else {
        float t = (gray - 0.5) * 2.0;
        FragColor = vec4(t, 0.0, 1.0 - t, 1.0);
    }
}