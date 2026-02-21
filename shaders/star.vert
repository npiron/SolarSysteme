#version 300 es
precision highp float;

layout(location = 0) in vec3 a_position;
layout(location = 1) in float a_brightness;

uniform mat4 u_view;
uniform mat4 u_projection;
uniform float u_time;

out float v_brightness;

void main() {
    v_brightness = a_brightness * (0.7 + 0.3 * sin(u_time * 2.0 + a_brightness * 100.0));
    gl_Position = u_projection * u_view * vec4(a_position, 1.0);
    gl_PointSize = max(1.0, a_brightness * 3.0);
}
