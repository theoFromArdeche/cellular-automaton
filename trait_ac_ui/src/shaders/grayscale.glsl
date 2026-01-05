#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 FragColor;
uniform sampler2D u_texture;

void main() {
    float gray = texture(u_texture, v_tc).r;
    FragColor = vec4(gray, gray, gray, 1.0);
}

