#version 300 es
precision highp float;

layout(location = 0) in vec3 a_position;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_pos;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);
    v_pos = a_position;
    gl_Position = u_projection * u_view * world_pos;
}
