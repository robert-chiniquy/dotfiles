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

// Curved ray - traces along ray and applies sinusoidal bend perpendicular to direction
float curvedRay(float2 origin, float2 dir, float2 coord, float width, float seedA, float seedB, float phase) {
    float2 toCoord = coord - origin;
    float maxDist = 2.5;

    // Project coord onto ray direction to find "along" distance
    float along = dot(toCoord, dir);
    if (along < 0.0) return 0.0;  // Behind origin

    // Perpendicular direction
    float2 perp = float2(-dir.y, dir.x);

    // The key: compute where the CURVED ray would be at this "along" distance
    // Apply sinusoidal displacement that increases with distance
    float bendFreq = (seedA + seedB) * 0.8;
    float bendPhase = along * bendFreq + phase;
    // S-curve bending - amplitude increases with distance squared (reduced 30%)
    float bendAmount = sin(bendPhase) * 0.42 * along * along;
    bendAmount += cos(bendPhase * 0.7 + 1.0) * 0.21 * along * along;

    // Where the curved ray center is at this distance
    float2 curvedRayPos = origin + dir * along + perp * bendAmount;

    // Distance from coord to the curved ray center
    float perpDist = length(coord - curvedRayPos);

    // Gaussian width falloff from curved ray center
    float widthFalloff = exp(-perpDist * perpDist / (width * width));

    // Distance fade
    float dist = length(toCoord);
    float distFade = 1.0 - smoothstep(0.0, maxDist, dist);

    return widthFalloff * distFade;
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

    // Use window seed for subtle variation without flip/rotation
    // (removed flip/rotation - was causing oscillation artifacts)
    float seedHash = fract(sin(windowSeed * 12.9898) * 43758.5453);

    // Semi-random jitter: varies timing by position and window (0.7x to 1.3x speed)
    float jitter = 0.7 + 0.6 * hash(float2(floor(uv.x * 8.0), floor(uv.y * 8.0) + seedHash));
    float jitteredTime = time * jitter;

    float phase = jitteredTime * 0.0000000067;  // Glacial ray animation with jitter (3x slower)

    // Quick access colors
    float3 deepPurple = float3(0.4, 0.1, 0.6);
    float3 neonPurple = float3(0.667, 0.376, 0.929);

    // Gradual ray appearance - each ray fades in over time
    float rayTime = time * 0.0000033;  // 300x slower fade-in
    float rayInterval = 3.0;  // 30 seconds between each ray appearing

    // Ray width grows over time
    float widthScale = 1.0 + min(rayTime / 60.0, 2.0);  // Slower growth

    // Reduced to 8 rays for lower CPU usage
    float rays = 0.0;

    // Bottom-left source
    float2 origin1 = float2(-0.85, -1.3);
    rays += curvedRay(origin1, normalize(float2(0.65, 1.0)), coord, 0.022 * widthScale, 1.3, 2.1, phase) * smoothstep(0.0, 1.0, rayTime - rayInterval * 0.0);
    rays += curvedRay(origin1, normalize(float2(0.95, 0.75)), coord, 0.020 * widthScale, 1.7, 1.6, phase + 1.0) * smoothstep(0.0, 1.0, rayTime - rayInterval * 1.0);

    // Center source
    float2 origin2 = float2(0.15, -1.5);
    rays += curvedRay(origin2, normalize(float2(-0.3, 1.0)), coord, 0.021 * widthScale, 1.6, 1.9, phase + 0.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 2.0);
    rays += curvedRay(origin2, normalize(float2(0.5, 0.95)), coord, 0.019 * widthScale, 1.2, 2.2, phase + 1.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 3.0);

    // Right source
    float2 origin3 = float2(0.7, -0.9);
    rays += curvedRay(origin3, normalize(float2(-0.8, 1.0)), coord, 0.020 * widthScale, 1.4, 2.0, phase + 0.15) * smoothstep(0.0, 1.0, rayTime - rayInterval * 4.0);
    rays += curvedRay(origin3, normalize(float2(-0.5, 0.8)), coord, 0.019 * widthScale, 1.7, 1.8, phase + 2.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 5.0);

    // Corner source
    float2 origin4 = float2(1.2, -1.4);
    rays += curvedRay(origin4, normalize(float2(-0.7, 0.9)), coord, 0.019 * widthScale, 1.4, 1.8, phase + 0.35) * smoothstep(0.0, 1.0, rayTime - rayInterval * 6.0);
    rays += curvedRay(origin4, normalize(float2(-0.9, 0.6)), coord, 0.020 * widthScale, 1.7, 1.6, phase + 1.85) * smoothstep(0.0, 1.0, rayTime - rayInterval * 7.0);

    rays = clamp(rays * 0.8, 0.0, 1.0);

    // === ACTUAL PURPLE DETECTION FROM WINDOW CONTENT ===
    float purpleActive = 0.0;
    constexpr sampler texSampler(mag_filter::linear, min_filter::linear);

    if (hasTexture > 0.5) {
        // Flip Y for CGImage coordinate system (origin at top-left)
        float2 texUV = float2(uv.x, 1.0 - uv.y);

        // Sample the underlying window texture
        float3 windowColor = windowTex.sample(texSampler, texUV).rgb;

        // Detect purple at this pixel
        float localPurple = detectPurple(windowColor);

        // Simplified detection - mostly local, minimal neighbor influence
        float2 texelSize = float2(1.0 / 1920.0, 1.0 / 1080.0);
        float neighborPurple = 0.0;
        neighborPurple += detectPurple(windowTex.sample(texSampler, texUV + float2(-texelSize.x * 5.0, 0.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, texUV + float2(texelSize.x * 5.0, 0.0)).rgb);
        neighborPurple *= 0.5;

        // Local dominates, neighbor just softens edges (reduced from 0.8 to 0.3)
        purpleActive = max(localPurple, neighborPurple * 0.3);

        // Moderate sensitivity
        purpleActive = smoothstep(0.10, 0.30, purpleActive);
    }

    // === PALETTE ROTATION WITH INTERPOLATION (from original shader) ===

    // Randomize stripe width using slow-changing hash - narrower bands
    float stripeHash = fract(sin(floor(time * 0.00000033) * 12.9898) * 43758.5453);  // 150x slower change
    float stripeWidthMult = 1.5 + stripeHash * 2.5;  // Higher = narrower bands

    // Wave distortion for organic movement - glacially slow
    float waveDistortion = sin(uv.x * 6.28318 + time * 0.000000017) * 0.08 +
                           cos(uv.y * 4.71239 + time * 0.000000027) * 0.05;

    // Spatial phase - color varies across screen + glacially slow time rotation
    // Start offset at 1.5 (between gold and cyan, far from purple at 6-7)
    float spatialPhase = (uv.x + uv.y * 0.7 + waveDistortion) * stripeWidthMult + time * 0.0000000067 + 1.5;
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
        float pulsePhase = time * 0.000000067;  // 300x slower
        float spatialOffset = (uv.x * 2.0 + uv.y * 3.0) * 6.28318;
        float pulse = sin(pulsePhase * 6.28318 + spatialOffset) * 0.5 + 0.5;
        pulse = pulse * pulse * pulse;  // Sharpen
        float transitionDepth = smoothstep(transitionStart, 1.0, fract(continuousIndex));
        rayColor *= (1.0 + pulse * transitionDepth * 2.5);  // 4x stronger pulse
    }

    // Global luminosity pulse (always active, not just during transitions)
    float globalPulse = sin(time * 0.0000001 + uv.y * 4.0) * 0.5 + 0.5;
    globalPulse = globalPulse * globalPulse;
    rayColor *= (1.0 + globalPulse * 0.4);

    // Shift toward purple when purple is active - STRONG tint (100000x sensitivity)
    rayColor = mix(rayColor, neonPurple, purpleActive * 0.8);

    // Purple reactive aura - INTENSE glow when purple detected
    float auraPhase = time * 0.000000033 + uv.y * 2.0;  // 1200x slower (4x reduction)
    float aura = sin(auraPhase) * 0.5 + 0.5;
    aura = aura * aura;
    float3 purpleAura = deepPurple * aura * purpleActive * 1.5;  // Strong aura when purple detected

    // Base tint - slightly more purple
    float3 baseTint = float3(0.05, 0.02, 0.10);

    // Combine (scanlines added later as overlay)
    float3 finalColor = baseTint + rays * rayColor * 0.95 + purpleAura;

    // Fade toward top
    finalColor *= 0.3 + (1.0 - uv.y) * 0.7;

    // Dark grey to black crush - aggressively push medium/dark grays to black
    float luminance = dot(finalColor, float3(0.299, 0.587, 0.114));
    float saturation = max(max(finalColor.r, finalColor.g), finalColor.b) -
                       min(min(finalColor.r, finalColor.g), finalColor.b);
    // Detect dark/medium greys: expanded luminance range, low saturation
    float isDarkGrey = (1.0 - smoothstep(0.0, 0.30, luminance)) * (1.0 - smoothstep(0.0, 0.15, saturation));
    // Strong crush towards black
    finalColor *= (1.0 - isDarkGrey * 0.95);

    // === SCANLINE GLITCH OVERLAY (applied after all color processing) ===
    // Only triggers when: purple detected AND ray covering AND random chance
    float glitchZone = purpleActive * step(0.1, rays);

    if (glitchZone > 0.01) {
        // Slow time quantization for sporadic glitches with jitter
        float glitchJitter = 0.6 + 0.8 * hash(float2(floor(uv.y * 20.0), seedHash));
        float glitchTime = floor(jitteredTime * 0.00000042 * glitchJitter);  // 1/160 speed (10x slower)

        // Variable scanline heights (1-20 pixels) AND variable gap heights (5-30 pixels)
        float baseY = uv.y * 400.0;
        float lineHeight = 1.0 + floor(hash(float2(floor(baseY * 0.05), glitchTime)) * 20.0);
        float gapHeight = 5.0 + floor(hash(float2(floor(baseY * 0.03), glitchTime + 7.0)) * 25.0);
        float scan1 = floor(baseY / (lineHeight + gapHeight));
        float withinLine = fract(baseY / (lineHeight + gapHeight)) < (lineHeight / (lineHeight + gapHeight));

        // Diagonal scanlines with variable line/gap heights
        float diagBase = (uv.x * 0.5 + uv.y) * 300.0;
        float diagLineHeight = 1.0 + floor(hash(float2(floor(diagBase * 0.05), glitchTime + 1.0)) * 15.0);
        float diagGapHeight = 5.0 + floor(hash(float2(floor(diagBase * 0.03), glitchTime + 8.0)) * 20.0);
        float diagScan = floor(diagBase / (diagLineHeight + diagGapHeight));
        float withinDiagLine = fract(diagBase / (diagLineHeight + diagGapHeight)) < (diagLineHeight / (diagLineHeight + diagGapHeight));

        // Random per scanline per time slice
        float r1 = hash(float2(glitchTime, scan1));
        float rd = hash(float2(glitchTime + 0.5, diagScan));

        // Glitch only within line regions, sparse
        float glitchChance = step(0.90, r1) * withinLine + step(0.92, rd) * withinDiagLine * 0.5;

        // Overlay scanlines on top of final color
        float glitchStrength = glitchChance * glitchZone * 0.25;
        finalColor += neonPurple * glitchStrength;
    }

    // Alpha - based on ray intensity with strong purple boost
    float alpha = (0.2 + rays * 0.5 + purpleActive * 0.4) * opacity;

    return float4(finalColor, alpha);
}
