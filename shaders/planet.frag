#version 300 es
precision highp float;

in vec3 v_normal;
in vec3 v_frag_pos;
in vec2 v_uv;

uniform vec3 u_color;
uniform vec3 u_light_pos;
uniform vec3 u_view_pos;
uniform bool u_is_star;
uniform bool u_has_texture;
uniform sampler2D u_texture;

out vec4 frag_color;

void main() {
    // Base color: texture if available, otherwise uniform color
    vec3 base_color = u_has_texture ? texture(u_texture, v_uv).rgb : u_color;

    if (u_is_star) {
        // Sun: self-illuminated with subtle surface detail
        frag_color = vec4(base_color, 1.0);
        return;
    }

    vec3 norm = normalize(v_normal);
    vec3 light_dir = normalize(u_light_pos - v_frag_pos);

    // Ambient
    float ambient_strength = 0.08;
    vec3 ambient = ambient_strength * base_color;

    // Diffuse (Lambertian)
    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = diff * base_color;

    // Specular (Blinn-Phong)
    vec3 view_dir = normalize(u_view_pos - v_frag_pos);
    vec3 halfway = normalize(light_dir + view_dir);
    float spec = pow(max(dot(norm, halfway), 0.0), 32.0);
    vec3 specular = vec3(0.15) * spec;

    // Atmosphere rim effect
    float rim = 1.0 - max(dot(norm, view_dir), 0.0);
    vec3 rim_color = base_color * 0.3 * pow(rim, 3.0);

    vec3 result = ambient + diffuse + specular + rim_color;
    frag_color = vec4(result, 1.0);
}
