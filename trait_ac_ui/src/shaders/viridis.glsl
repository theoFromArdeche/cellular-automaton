#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 FragColor;
uniform sampler2D u_texture;

vec3 viridis(float t) {
    const vec3 c0 = vec3(0.267004, 0.004874, 0.329415);
    const vec3 c1 = vec3(0.282623, 0.140926, 0.457517);
    const vec3 c2 = vec3(0.253935, 0.265254, 0.529983);
    const vec3 c3 = vec3(0.206756, 0.371758, 0.553117);
    const vec3 c4 = vec3(0.163625, 0.471133, 0.558148);
    const vec3 c5 = vec3(0.127568, 0.566949, 0.550556);
    const vec3 c6 = vec3(0.134692, 0.658636, 0.517649);
    const vec3 c7 = vec3(0.266941, 0.748751, 0.440573);
    const vec3 c8 = vec3(0.477504, 0.821444, 0.318195);
    const vec3 c9 = vec3(0.741388, 0.873449, 0.149561);
    const vec3 c10 = vec3(0.993248, 0.906157, 0.143936);
    
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
    FragColor = vec4(viridis(gray), 1.0);
}

