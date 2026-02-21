#version 300 es
precision highp float;

layout(location = 0) in vec3 a_position;
layout(location = 1) in float a_alpha;

uniform mat4 u_view;
uniform mat4 u_projection;

out float v_alpha;

void main() {
    v_alpha = a_alpha;
    gl_Position = u_projection * u_view * vec4(a_position, 1.0);
}
