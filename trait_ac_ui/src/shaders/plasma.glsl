#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 FragColor;
uniform sampler2D u_texture;
uniform float u_base_color;

vec3 plasma(float t) {
    const vec3 c0 = vec3(0.050383, 0.029803, 0.527975);
    const vec3 c1 = vec3(0.180653, 0.018803, 0.589071);
    const vec3 c2 = vec3(0.299471, 0.007633, 0.633583);
    const vec3 c3 = vec3(0.420397, 0.004816, 0.658390);
    const vec3 c4 = vec3(0.537158, 0.047331, 0.654670);
    const vec3 c5 = vec3(0.645293, 0.100836, 0.628397);
    const vec3 c6 = vec3(0.741388, 0.155158, 0.586606);
    const vec3 c7 = vec3(0.825830, 0.211364, 0.530398);
    const vec3 c8 = vec3(0.898692, 0.270914, 0.461779);
    const vec3 c9 = vec3(0.959988, 0.339161, 0.378610);
    const vec3 c10 = vec3(0.940015, 0.975158, 0.131326);
    
    float x = clamp(t, 0.0, 1.0) * 10.0;
    int i = int(floor(x));
    float f = fract(x);
    
    if (i == 0) return mix(c0, c1, f);
    if (i == 1) return mix(c1, c2, f);
    if (i == 2) return mix(c2, c3, f);
    if (i == 3) return mix(c3, c4, f);
    if (i == 4) return mix(c4, c5, f);
    if (i == 5) return mix(c5, c6, f);
    if (i == 6) return mix(c6, c7, f);
    if (i == 7) return mix(c7, c8, f);
    if (i == 8) return mix(c8, c9, f);
    if (i == 9) return mix(c9, c10, f);
    return c10;
}

void main() {
    float gray = texture(u_texture, v_tc).r;
    FragColor = vec4(plasma(gray), 1.0);
}
