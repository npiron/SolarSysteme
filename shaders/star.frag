#version 300 es
precision highp float;

in float v_brightness;

out vec4 frag_color;

void main() {
    // Soft circular point
    vec2 coord = gl_PointCoord - vec2(0.5);
    float dist = length(coord);
    if (dist > 0.5) discard;
    float alpha = (1.0 - dist * 2.0) * v_brightness;
    frag_color = vec4(vec3(0.9, 0.92, 1.0) * v_brightness, alpha);
}
