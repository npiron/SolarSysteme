#version 300 es
precision highp float;

uniform vec3 u_color;

in float v_alpha;
out vec4 frag_color;

void main() {
    frag_color = vec4(u_color, v_alpha * 0.6);
}
