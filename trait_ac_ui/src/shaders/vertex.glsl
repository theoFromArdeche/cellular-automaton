#version 300 es
precision highp float;

in vec2 a_pos;
in vec2 a_tc;

out vec2 v_tc;

uniform vec2 u_rect_min;
uniform vec2 u_rect_size;

void main() {
    // convert from absolute egui coords -> rect-local
    vec2 local = a_pos - u_rect_min;

    // normalize
    vec2 pos = local / u_rect_size;
    pos = pos * 2.0 - 1.0;
    pos.y = -pos.y;

    gl_Position = vec4(pos, 0.0, 1.0);
    v_tc = a_tc;
}
