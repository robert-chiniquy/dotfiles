// Vaporwave Overlay Shader - curved light rays with glitch
#include <metal_stdlib>
using namespace metal;

struct VertexOut {
    float4 position [[position]];
    float2 uv;
};

// Hash for glitch noise
float hash(float2 p) {
    p = fract(p * float2(234.34, 435.345));
    p += dot(p, p + 34.23);
    return fract(p.x * p.y);
}

// Curved ray matching original shader's bend logic (underwater-glitchy.glsl)
float curvedRay(float2 origin, float2 dir, float2 coord, float width, float seedA, float seedB, float phase) {
    float2 toCoord = coord - origin;
    float dist = length(toCoord);
    float maxDist = 2.5;
    float distRatio = clamp(dist / maxDist, 0.0, 1.0);

    // Create perpendicular for bending (like original)
    float2 perp = float2(-toCoord.y, toCoord.x);
    perp = normalize(perp + 0.0001);

    // STRONG bending - original uses 280/120 pixel values, scaled to normalized coords
    // Frequency based on distance (original: distance * 0.012 * seeds)
    // In normalized coords where dist ~0-2.5, scale up frequency
    float combinedFreq = dist * 3.0 * (seedA + seedB) * 0.5 + phase;
    // Bend amounts scaled for -1 to 1 coord space (original 280/120 pixels on ~1000px screen)
    float bendAmount = sin(combinedFreq) * 0.7 + cos(combinedFreq * 1.7) * 0.35;
    bendAmount *= distRatio * distRatio;  // Increases with distance from source

    // Apply bend to get curved coordinate
    float2 bentCoord = coord + perp * bendAmount;
    float2 bentDir = normalize(bentCoord - origin);

    // Ray intensity based on angle alignment with bent direction
    float cosAngle = dot(bentDir, dir);
    float rayPhase = cosAngle * (seedA + seedB) * 0.5 + phase * 0.3;
    float ray = 0.5 + 0.5 * sin(rayPhase * 8.0);

    // Soft threshold - creates defined ray edges
    ray = smoothstep(0.4, 0.9, ray);

    // Gaussian width falloff perpendicular to ray direction
    float along = dot(toCoord, dir);
    if (along < 0.0) return 0.0;
    float2 proj = origin + dir * along;
    float perpDist = length(coord - proj);
    float widthFalloff = exp(-perpDist * perpDist / (width * width));

    // Distance fade
    float distFade = 1.0 - smoothstep(0.0, maxDist, dist);

    return ray * widthFalloff * distFade;
}

vertex VertexOut vaporwave_vertex(uint vid [[vertex_id]]) {
    VertexOut out;
    float2 positions[] = {
        float2(-1.0, -1.0),
        float2( 1.0, -1.0),
        float2(-1.0,  1.0),
        float2( 1.0,  1.0)
    };
    out.position = float4(positions[vid], 0.0, 1.0);
    out.uv = positions[vid] * 0.5 + 0.5;
    return out;
}

// Vaporwave palette - matches original shader exactly
constant float3 VAPORWAVE_PALETTE[] = {
    float3(1.00, 0.00, 0.973),   // 0: Hot Magenta
    float3(0.984, 0.718, 0.145), // 1: Warm Gold
    float3(0.361, 0.925, 1.00),  // 2: Electric Cyan
    float3(1.00, 0.00, 0.973),   // 3: Hot Magenta
    float3(0.361, 0.925, 1.00),  // 4: Electric Cyan
    float3(0.361, 0.925, 1.00),  // 5: Electric Cyan
    float3(0.753, 0.502, 0.816), // 6: Deep Purple
    float3(0.670, 0.376, 0.929)  // 7: Neon Purple
};

// Detect if a color is "purple-ish" - returns 0.0 to 1.0
float detectPurple(float3 color) {
    // Purple has high red, low green, high blue
    // Also detect magenta (high red, low green, high blue)
    float r = color.r;
    float g = color.g;
    float b = color.b;

    // Purple detection: red and blue both present, green is lower
    float redBlueMin = min(r, b);
    float greenDeficit = max(0.0, redBlueMin - g);

    // Stronger when red and blue are both high and green is low
    float purpleness = greenDeficit * (r + b) * 0.5;

    // Also detect violet/magenta tones
    float isMagenta = step(0.3, r) * step(0.3, b) * (1.0 - smoothstep(0.0, 0.5, g));

    return clamp(purpleness * 3.0 + isMagenta * 0.5, 0.0, 1.0);
}

fragment float4 vaporwave_fragment(
    VertexOut in [[stage_in]],
    constant float &time [[buffer(0)]],
    constant float &opacity [[buffer(1)]],
    constant float &hasTexture [[buffer(2)]],
    constant float &windowSeed [[buffer(3)]],
    texture2d<float> windowTex [[texture(0)]]
) {
    float2 uv = in.uv;
    float2 coord = uv * 2.0 - 1.0;

    // Random rotation and flip per window (cheap - just uses seed)
    float seedHash = fract(sin(windowSeed * 12.9898) * 43758.5453);
    float rotation = seedHash * 6.28318;  // 0 to 2*PI
    float flipX = step(0.5, fract(seedHash * 2.0)) * 2.0 - 1.0;  // -1 or 1
    float flipY = step(0.5, fract(seedHash * 3.0)) * 2.0 - 1.0;  // -1 or 1

    // Apply random transform to coordinates
    float cosR = cos(rotation);
    float sinR = sin(rotation);
    float2 rotatedCoord = float2(
        coord.x * cosR - coord.y * sinR,
        coord.x * sinR + coord.y * cosR
    ) * float2(flipX, flipY);
    coord = rotatedCoord;

    float phase = time * 0.000001;  // 30x slower ray animation

    // Quick access colors
    float3 deepPurple = float3(0.4, 0.1, 0.6);
    float3 neonPurple = float3(0.667, 0.376, 0.929);

    // Gradual ray appearance - each ray fades in over time
    // rayTime in seconds (time is in milliseconds)
    float rayTime = time * 0.001;
    float rayInterval = 2.0;  // 2 seconds between each ray appearing

    // Ray width grows over time: starts at base, grows to 3x over 60 seconds
    float widthScale = 1.0 + min(rayTime / 30.0, 2.0);  // 1.0 to 3.0

    // Curved rays from multiple ASYMMETRIC sources - 18 total, skewed origins and angles
    float rays = 0.0;

    // Bottom-left source (skewed far left, below screen)
    float2 origin1 = float2(-0.85, -1.3);
    rays += curvedRay(origin1, normalize(float2(0.65, 1.0)), coord, 0.018 * widthScale, 1.3, 2.1, phase) * smoothstep(0.0, 1.0, rayTime - rayInterval * 0.0);
    rays += curvedRay(origin1, normalize(float2(0.82, 0.9)), coord, 0.016 * widthScale, 1.5, 1.8, phase + 0.5) * smoothstep(0.0, 1.0, rayTime - rayInterval * 1.0);
    rays += curvedRay(origin1, normalize(float2(0.95, 0.75)), coord, 0.015 * widthScale, 1.7, 1.6, phase + 1.0) * smoothstep(0.0, 1.0, rayTime - rayInterval * 2.0);
    rays += curvedRay(origin1, normalize(float2(0.4, 1.0)), coord, 0.017 * widthScale, 1.4, 2.0, phase + 1.5) * smoothstep(0.0, 1.0, rayTime - rayInterval * 3.0);

    // Off-center source (slightly right of center, very low)
    float2 origin2 = float2(0.15, -1.5);
    rays += curvedRay(origin2, normalize(float2(-0.3, 1.0)), coord, 0.017 * widthScale, 1.6, 1.9, phase + 0.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 4.0);
    rays += curvedRay(origin2, normalize(float2(0.1, 1.0)), coord, 0.016 * widthScale, 1.8, 1.7, phase + 0.75) * smoothstep(0.0, 1.0, rayTime - rayInterval * 5.0);
    rays += curvedRay(origin2, normalize(float2(0.5, 0.95)), coord, 0.015 * widthScale, 1.2, 2.2, phase + 1.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 6.0);
    rays += curvedRay(origin2, normalize(float2(-0.6, 0.85)), coord, 0.018 * widthScale, 1.9, 1.5, phase + 1.75) * smoothstep(0.0, 1.0, rayTime - rayInterval * 7.0);

    // Far right source (skewed right, higher up)
    float2 origin3 = float2(0.7, -0.9);
    rays += curvedRay(origin3, normalize(float2(-0.8, 1.0)), coord, 0.016 * widthScale, 1.4, 2.0, phase + 0.15) * smoothstep(0.0, 1.0, rayTime - rayInterval * 8.0);
    rays += curvedRay(origin3, normalize(float2(-0.5, 0.8)), coord, 0.015 * widthScale, 1.7, 1.8, phase + 2.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 9.0);
    rays += curvedRay(origin3, normalize(float2(-0.95, 0.7)), coord, 0.014 * widthScale, 1.3, 2.0, phase + 2.15) * smoothstep(0.0, 1.0, rayTime - rayInterval * 10.0);

    // Extra skewed source (far left, mid-height - unusual position)
    float2 origin4 = float2(-1.1, -0.6);
    rays += curvedRay(origin4, normalize(float2(0.9, 0.5)), coord, 0.017 * widthScale, 1.6, 1.7, phase + 0.65) * smoothstep(0.0, 1.0, rayTime - rayInterval * 11.0);
    rays += curvedRay(origin4, normalize(float2(0.7, 0.8)), coord, 0.015 * widthScale, 1.5, 1.9, phase + 1.15) * smoothstep(0.0, 1.0, rayTime - rayInterval * 12.0);
    rays += curvedRay(origin4, normalize(float2(1.0, 0.3)), coord, 0.016 * widthScale, 1.8, 1.5, phase + 2.65) * smoothstep(0.0, 1.0, rayTime - rayInterval * 13.0);

    // Corner source (bottom right corner, shooting diagonally)
    float2 origin5 = float2(1.2, -1.4);
    rays += curvedRay(origin5, normalize(float2(-0.7, 0.9)), coord, 0.015 * widthScale, 1.4, 1.8, phase + 0.35) * smoothstep(0.0, 1.0, rayTime - rayInterval * 14.0);
    rays += curvedRay(origin5, normalize(float2(-0.9, 0.6)), coord, 0.016 * widthScale, 1.7, 1.6, phase + 1.85) * smoothstep(0.0, 1.0, rayTime - rayInterval * 15.0);
    rays += curvedRay(origin5, normalize(float2(-0.5, 1.0)), coord, 0.014 * widthScale, 1.3, 2.1, phase + 2.35) * smoothstep(0.0, 1.0, rayTime - rayInterval * 16.0);
    rays += curvedRay(origin5, normalize(float2(-1.0, 0.4)), coord, 0.017 * widthScale, 1.9, 1.4, phase + 0.95) * smoothstep(0.0, 1.0, rayTime - rayInterval * 17.0);

    rays = clamp(rays * 0.7, 0.0, 1.0);  // More visible rays

    // === ACTUAL PURPLE DETECTION FROM WINDOW CONTENT ===
    float purpleActive = 0.0;
    constexpr sampler texSampler(mag_filter::linear, min_filter::linear);

    if (hasTexture > 0.5) {
        // Sample the underlying window texture
        float3 windowColor = windowTex.sample(texSampler, uv).rgb;

        // Detect purple at this pixel
        float localPurple = detectPurple(windowColor);

        // Also sample nearby pixels for area detection (blur/spread the detection)
        float2 texelSize = float2(1.0 / 1920.0, 1.0 / 1080.0);  // Approximate
        float neighborPurple = 0.0;
        neighborPurple += detectPurple(windowTex.sample(texSampler, uv + float2(-texelSize.x * 3.0, 0.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, uv + float2(texelSize.x * 3.0, 0.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, uv + float2(0.0, -texelSize.y * 3.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, uv + float2(0.0, texelSize.y * 3.0)).rgb);
        neighborPurple *= 0.25;

        // Combine local and neighbor detection
        purpleActive = max(localPurple, neighborPurple * 0.8);

        // EXTREME sensitivity - any hint of purple triggers full response
        purpleActive = smoothstep(0.05, 0.15, purpleActive);
    }

    // === MOIRE SCANLINE EFFECT (low CPU) ===
    // 3 layers at prime frequencies - interference creates moire
    float glitchTime = floor(time * 0.00001);

    // Three scanline grids at prime frequencies
    float scan1 = floor(uv.y * 1997.0);
    float scan2 = floor(uv.y * 673.0);
    float diagScan = floor((uv.x * 0.6 + uv.y) * 1117.0);

    // Single hash per layer (reuse for presence check)
    float r1 = hash(float2(glitchTime, scan1));
    float r2 = hash(float2(glitchTime, scan2));
    float rd = hash(float2(glitchTime, diagScan));

    // Skip ~35% of scanlines (use same hash, different threshold)
    float present1 = step(0.35, r1);
    float present2 = step(0.30, r2);
    float presentD = step(0.45, rd);

    // Glitches concentrated near purple
    float glitchThreshold = mix(0.99, 0.1, purpleActive);

    // Combine layers - moire from interference
    float glitchStrength = step(glitchThreshold, r1) * present1 * 0.35 +
                           step(glitchThreshold, r2) * present2 * 0.35 +
                           step(glitchThreshold, rd) * presentD * 0.3;

    // Purple scanline glitch
    float3 glitchColor = neonPurple * glitchStrength * (0.1 + purpleActive * 0.5);

    // === PALETTE ROTATION WITH INTERPOLATION (from original shader) ===

    // Randomize stripe width using slow-changing hash - narrower bands
    float stripeHash = fract(sin(floor(time * 0.000001) * 12.9898) * 43758.5453);  // 50x slower change
    float stripeWidthMult = 1.5 + stripeHash * 2.5;  // Higher = narrower bands

    // Wave distortion for organic movement - glacially slow
    float waveDistortion = sin(uv.x * 6.28318 + time * 0.00000005) * 0.08 +
                           cos(uv.y * 4.71239 + time * 0.00000008) * 0.05;

    // Spatial phase - color varies across screen + glacially slow time rotation
    // Start offset at 1.5 (between gold and cyan, far from purple at 6-7)
    float spatialPhase = (uv.x + uv.y * 0.7 + waveDistortion) * stripeWidthMult + time * 0.00000002 + 1.5;
    float continuousIndex = fmod(spatialPhase, 8.0);
    int colorIndex = int(floor(continuousIndex));
    float colorBlend = fract(continuousIndex);

    // Easing: hold on pure colors, transition only in small intervals
    float transitionHash = fract(sin(float(colorIndex) * 12.9898) * 43758.5453);
    float transitionStart = 0.88 + transitionHash * 0.08;  // 88-96%

    float isTransitioning = 0.0;
    if (colorBlend < transitionStart) {
        colorBlend = 0.0;  // Hold pure color
    } else {
        isTransitioning = 1.0;
        float normalizedT = (colorBlend - transitionStart) / (1.0 - transitionStart);
        colorBlend = normalizedT * normalizedT * (3.0 - 2.0 * normalizedT);  // Smoothstep
    }

    // Get colors from palette and interpolate
    float3 paletteColor1 = VAPORWAVE_PALETTE[colorIndex % 8];
    float3 paletteColor2 = VAPORWAVE_PALETTE[(colorIndex + 1) % 8];
    float3 rayColor = mix(paletteColor1, paletteColor2, colorBlend);

    // Pulse during transitions - INTENSE luminosity
    if (isTransitioning > 0.5) {
        float pulsePhase = time * 0.0000002;  // 100x slower
        float spatialOffset = (uv.x * 2.0 + uv.y * 3.0) * 6.28318;
        float pulse = sin(pulsePhase * 6.28318 + spatialOffset) * 0.5 + 0.5;
        pulse = pulse * pulse * pulse;  // Sharpen
        float transitionDepth = smoothstep(transitionStart, 1.0, fract(continuousIndex));
        rayColor *= (1.0 + pulse * transitionDepth * 2.5);  // 4x stronger pulse
    }

    // Global luminosity pulse (always active, not just during transitions)
    float globalPulse = sin(time * 0.0000003 + uv.y * 4.0) * 0.5 + 0.5;
    globalPulse = globalPulse * globalPulse;
    rayColor *= (1.0 + globalPulse * 0.4);

    // Shift toward purple when purple is active - STRONG tint (100000x sensitivity)
    rayColor = mix(rayColor, neonPurple, purpleActive * 0.8);

    // Purple reactive aura - INTENSE glow when purple detected
    float auraPhase = time * 0.0000004 + uv.y * 2.0;  // 100x slower
    float aura = sin(auraPhase) * 0.5 + 0.5;
    aura = aura * aura;
    float3 purpleAura = deepPurple * aura * purpleActive * 1.5;  // Strong aura when purple detected

    // Base tint - slightly more purple
    float3 baseTint = float3(0.05, 0.02, 0.10);

    // Combine
    float3 finalColor = baseTint + rays * rayColor * 0.95 + purpleAura + glitchColor;

    // Fade toward top
    finalColor *= 0.3 + (1.0 - uv.y) * 0.7;

    // Alpha - based on ray intensity with strong purple boost
    float alpha = (0.2 + rays * 0.5 + purpleActive * 0.4) * opacity;

    return float4(finalColor, alpha);
}
