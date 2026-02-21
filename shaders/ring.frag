#version 300 es
precision highp float;

in vec3 v_pos;

uniform vec3 u_color;

out vec4 frag_color;

void main() {
    float dist = length(v_pos.xz);
    float alpha = smoothstep(1.2, 1.4, dist) * (1.0 - smoothstep(2.2, 2.4, dist));
    frag_color = vec4(u_color, alpha * 0.5);
}
