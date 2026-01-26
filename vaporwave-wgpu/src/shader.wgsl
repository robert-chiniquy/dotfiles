// Vaporwave Overlay Shader - WGSL port from Metal

struct Uniforms {
    time: f32,
    opacity: f32,
    window_seed: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Hash for glitch noise
fn hash(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(234.34, 435.345));
    q = q + dot(q, q + 34.23);
    return fract(q.x * q.y);
}

// Curved ray - traces along ray with sinusoidal bend
fn curved_ray(origin: vec2<f32>, dir: vec2<f32>, coord: vec2<f32>, width: f32, seed_a: f32, seed_b: f32, phase: f32) -> f32 {
    let to_coord = coord - origin;
    let max_dist = 2.5;

    let along = dot(to_coord, dir);
    if (along < 0.0) { return 0.0; }

    let perp = vec2<f32>(-dir.y, dir.x);

    let bend_freq = (seed_a + seed_b) * 0.8;
    let bend_phase = along * bend_freq + phase;
    var bend_amount = sin(bend_phase) * 0.42 * along * along;
    bend_amount = bend_amount + cos(bend_phase * 0.7 + 1.0) * 0.21 * along * along;

    let curved_ray_pos = origin + dir * along + perp * bend_amount;
    let perp_dist = length(coord - curved_ray_pos);
    let width_falloff = exp(-perp_dist * perp_dist / (width * width));

    let dist = length(to_coord);
    let dist_fade = 1.0 - smoothstep(0.0, max_dist, dist);

    return width_falloff * dist_fade;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, 1.0)
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = positions[vertex_index] * 0.5 + 0.5;
    return out;
}

// Vaporwave palette
const PALETTE: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(1.00, 0.00, 0.973),   // Hot Magenta
    vec3<f32>(0.984, 0.718, 0.145), // Warm Gold
    vec3<f32>(0.361, 0.925, 1.00),  // Electric Cyan
    vec3<f32>(1.00, 0.00, 0.973),   // Hot Magenta
    vec3<f32>(0.361, 0.925, 1.00),  // Electric Cyan
    vec3<f32>(0.361, 0.925, 1.00),  // Electric Cyan
    vec3<f32>(0.753, 0.502, 0.816), // Deep Purple
    vec3<f32>(0.670, 0.376, 0.929)  // Neon Purple
);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let coord = uv * 2.0 - 1.0;
    let time = uniforms.time;

    let seed_hash = fract(sin(uniforms.window_seed * 12.9898) * 43758.5453);

    // Semi-random jitter
    let jitter = 0.7 + 0.6 * hash(vec2<f32>(floor(uv.x * 8.0), floor(uv.y * 8.0) + seed_hash));
    let jittered_time = time * jitter;

    let phase = jittered_time * 0.0000000067;

    let deep_purple = vec3<f32>(0.4, 0.1, 0.6);
    let neon_purple = vec3<f32>(0.667, 0.376, 0.929);

    let ray_time = time * 0.0000033;
    let ray_interval = 3.0;
    let width_scale = 1.0 + min(ray_time / 60.0, 2.0);

    var rays = 0.0;

    // Bottom-left source
    let origin1 = vec2<f32>(-0.85, -1.3);
    rays = rays + curved_ray(origin1, normalize(vec2<f32>(0.65, 1.0)), coord, 0.022 * width_scale, 1.3, 2.1, phase) * smoothstep(0.0, 1.0, ray_time - ray_interval * 0.0);
    rays = rays + curved_ray(origin1, normalize(vec2<f32>(0.95, 0.75)), coord, 0.020 * width_scale, 1.7, 1.6, phase + 1.0) * smoothstep(0.0, 1.0, ray_time - ray_interval * 1.0);

    // Center source
    let origin2 = vec2<f32>(0.15, -1.5);
    rays = rays + curved_ray(origin2, normalize(vec2<f32>(-0.3, 1.0)), coord, 0.021 * width_scale, 1.6, 1.9, phase + 0.25) * smoothstep(0.0, 1.0, ray_time - ray_interval * 2.0);
    rays = rays + curved_ray(origin2, normalize(vec2<f32>(0.5, 0.95)), coord, 0.019 * width_scale, 1.2, 2.2, phase + 1.25) * smoothstep(0.0, 1.0, ray_time - ray_interval * 3.0);

    // Right source
    let origin3 = vec2<f32>(0.7, -0.9);
    rays = rays + curved_ray(origin3, normalize(vec2<f32>(-0.8, 1.0)), coord, 0.020 * width_scale, 1.4, 2.0, phase + 0.15) * smoothstep(0.0, 1.0, ray_time - ray_interval * 4.0);
    rays = rays + curved_ray(origin3, normalize(vec2<f32>(-0.5, 0.8)), coord, 0.019 * width_scale, 1.7, 1.8, phase + 2.25) * smoothstep(0.0, 1.0, ray_time - ray_interval * 5.0);

    // Corner source
    let origin4 = vec2<f32>(1.2, -1.4);
    rays = rays + curved_ray(origin4, normalize(vec2<f32>(-0.7, 0.9)), coord, 0.019 * width_scale, 1.4, 1.8, phase + 0.35) * smoothstep(0.0, 1.0, ray_time - ray_interval * 6.0);
    rays = rays + curved_ray(origin4, normalize(vec2<f32>(-0.9, 0.6)), coord, 0.020 * width_scale, 1.7, 1.6, phase + 1.85) * smoothstep(0.0, 1.0, ray_time - ray_interval * 7.0);

    rays = clamp(rays * 0.8, 0.0, 1.0);

    // Palette rotation
    let stripe_hash = fract(sin(floor(time * 0.00000033) * 12.9898) * 43758.5453);
    let stripe_width_mult = 1.5 + stripe_hash * 2.5;

    let wave_distortion = sin(uv.x * 6.28318 + time * 0.000000017) * 0.08 +
                          cos(uv.y * 4.71239 + time * 0.000000027) * 0.05;

    let spatial_phase = (uv.x + uv.y * 0.7 + wave_distortion) * stripe_width_mult + time * 0.0000000067 + 1.5;
    let continuous_index = spatial_phase % 8.0;
    let color_index = i32(floor(continuous_index));
    var color_blend = fract(continuous_index);

    let transition_hash = fract(sin(f32(color_index) * 12.9898) * 43758.5453);
    let transition_start = 0.88 + transition_hash * 0.08;

    var is_transitioning = 0.0;
    if (color_blend < transition_start) {
        color_blend = 0.0;
    } else {
        is_transitioning = 1.0;
        let normalized_t = (color_blend - transition_start) / (1.0 - transition_start);
        color_blend = normalized_t * normalized_t * (3.0 - 2.0 * normalized_t);
    }

    let palette_color1 = PALETTE[color_index % 8];
    let palette_color2 = PALETTE[(color_index + 1) % 8];
    var ray_color = mix(palette_color1, palette_color2, color_blend);

    // Global pulse
    let global_pulse = sin(time * 0.0000001 + uv.y * 4.0) * 0.5 + 0.5;
    ray_color = ray_color * (1.0 + global_pulse * global_pulse * 0.4);

    // Base tint
    let base_tint = vec3<f32>(0.05, 0.02, 0.10);

    // Combine
    var final_color = base_tint + rays * ray_color * 0.95;

    // Fade toward top
    final_color = final_color * (0.3 + (1.0 - uv.y) * 0.7);

    // Dark grey to black crush
    let luminance = dot(final_color, vec3<f32>(0.299, 0.587, 0.114));
    let saturation = max(max(final_color.r, final_color.g), final_color.b) -
                     min(min(final_color.r, final_color.g), final_color.b);
    let is_dark_grey = (1.0 - smoothstep(0.0, 0.30, luminance)) * (1.0 - smoothstep(0.0, 0.15, saturation));
    final_color = final_color * (1.0 - is_dark_grey * 0.95);

    // Scanline glitch overlay
    let glitch_zone = step(0.1, rays);
    if (glitch_zone > 0.01) {
        let glitch_jitter = 0.6 + 0.8 * hash(vec2<f32>(floor(uv.y * 20.0), seed_hash));
        let glitch_time = floor(jittered_time * 0.00000042 * glitch_jitter);  // 10x slower

        let base_y = uv.y * 400.0;
        let line_height = 1.0 + floor(hash(vec2<f32>(floor(base_y * 0.05), glitch_time)) * 20.0);
        let gap_height = 5.0 + floor(hash(vec2<f32>(floor(base_y * 0.03), glitch_time + 7.0)) * 25.0);
        let scan1 = floor(base_y / (line_height + gap_height));
        let within_line = select(0.0, 1.0, fract(base_y / (line_height + gap_height)) < (line_height / (line_height + gap_height)));

        let r1 = hash(vec2<f32>(glitch_time, scan1));
        let glitch_chance = step(0.90, r1) * within_line;
        let glitch_strength = glitch_chance * glitch_zone * 0.25;
        final_color = final_color + neon_purple * glitch_strength;
    }

    let alpha = (0.2 + rays * 0.5) * uniforms.opacity;

    return vec4<f32>(final_color, alpha);
}
