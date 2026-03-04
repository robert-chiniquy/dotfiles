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

    // MEGA bending - wild wavy rays
    float combinedFreq = dist * 6.0 * (seedA + seedB) * 0.5 + phase;
    float bendAmount = sin(combinedFreq) * 6.0 + cos(combinedFreq * 1.7) * 3.5 + sin(combinedFreq * 2.3) * 2.0;
    bendAmount *= distRatio * distRatio;

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
// STRICT - only triggers on actual purple/magenta colors
float detectPurple(float3 color) {
    float r = color.r;
    float g = color.g;
    float b = color.b;

    // Core requirement: green must be lower than red AND blue
    float greenDeficit = min(r, b) - g;
    if (greenDeficit < 0.05) return 0.0;  // Looser threshold

    // Both red and blue must be present
    if (r < 0.15 || b < 0.15) return 0.0;  // Lower floor

    // Green must be lower than the average of red and blue
    float rbAvg = (r + b) * 0.5;
    if (g > rbAvg * 0.8) return 0.0;  // More permissive

    // Purpleness based on how much green deficit there is — boosted
    float purpleness = greenDeficit * (r + b) * 3.5;

    return clamp(purpleness, 0.0, 1.0);
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

    float phase = time * 0.0001;  // Fast ray animation for per-window mode

    // Quick access colors
    float3 deepPurple = float3(0.4, 0.1, 0.6);
    float3 neonPurple = float3(0.667, 0.376, 0.929);

    // Gradual ray appearance - each ray fades in over time
    // rayTime in seconds (time is in milliseconds)
    float rayTime = time * 0.001;
    float rayInterval = 0.3;  // 0.3 seconds between each ray appearing

    // Ray width grows over time: starts small, grows to 10x over 30 seconds
    float widthScale = 0.3 + min(rayTime / 3.0, 9.7);  // Full width in 3 sec

    // Ray intensity grows over time: starts at 0, hits 100% at 2 seconds
    float rayDelay = 0.5;  // Rays start 0.5 seconds after overlay appears
    float rayIntensityTime = max(0.0, rayTime - rayDelay);
    float intensityScale = min(rayIntensityTime / 2.0, 1.0);  // Full intensity in 2 sec

    // Purple effect fades in fast
    float purpleFadeIn = min(rayTime / 1.0, 1.0);  // Purple at 100% by 1 second
    float purpleTimeScale = purpleFadeIn;

    // Curved rays from 3 ASYMMETRIC sources - 10 total (CPU optimized)
    float rays = 0.0;

    // Bottom-left source (skewed far left, below screen)
    float2 origin1 = float2(-0.85, -1.3);
    rays += curvedRay(origin1, normalize(float2(0.65, 1.0)), coord, 0.018 * widthScale, 1.3, 2.1, phase) * smoothstep(0.0, 1.0, rayTime - rayInterval * 0.0);
    rays += curvedRay(origin1, normalize(float2(0.82, 0.9)), coord, 0.016 * widthScale, 1.5, 1.8, phase + 0.5) * smoothstep(0.0, 1.0, rayTime - rayInterval * 1.0);
    rays += curvedRay(origin1, normalize(float2(0.4, 1.0)), coord, 0.017 * widthScale, 1.4, 2.0, phase + 1.5) * smoothstep(0.0, 1.0, rayTime - rayInterval * 2.0);

    // Off-center source (slightly right of center, very low)
    float2 origin2 = float2(0.15, -1.5);
    rays += curvedRay(origin2, normalize(float2(-0.3, 1.0)), coord, 0.017 * widthScale, 1.6, 1.9, phase + 0.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 3.0);
    rays += curvedRay(origin2, normalize(float2(0.1, 1.0)), coord, 0.016 * widthScale, 1.8, 1.7, phase + 0.75) * smoothstep(0.0, 1.0, rayTime - rayInterval * 4.0);
    rays += curvedRay(origin2, normalize(float2(0.5, 0.95)), coord, 0.015 * widthScale, 1.2, 2.2, phase + 1.25) * smoothstep(0.0, 1.0, rayTime - rayInterval * 5.0);
    rays += curvedRay(origin2, normalize(float2(-0.6, 0.85)), coord, 0.018 * widthScale, 1.9, 1.5, phase + 1.75) * smoothstep(0.0, 1.0, rayTime - rayInterval * 6.0);

    // Corner source (bottom right corner, shooting diagonally)
    float2 origin3 = float2(1.2, -1.4);
    rays += curvedRay(origin3, normalize(float2(-0.7, 0.9)), coord, 0.015 * widthScale, 1.4, 1.8, phase + 0.35) * smoothstep(0.0, 1.0, rayTime - rayInterval * 7.0);
    rays += curvedRay(origin3, normalize(float2(-0.9, 0.6)), coord, 0.016 * widthScale, 1.7, 1.6, phase + 1.85) * smoothstep(0.0, 1.0, rayTime - rayInterval * 8.0);
    rays += curvedRay(origin3, normalize(float2(-0.5, 1.0)), coord, 0.014 * widthScale, 1.3, 2.1, phase + 2.35) * smoothstep(0.0, 1.0, rayTime - rayInterval * 9.0);

    rays = clamp(rays * 0.7 * intensityScale + 0.05, 0.0, 1.0);  // Subtle base + moderate intensity

    // === ACTUAL PURPLE DETECTION FROM WINDOW CONTENT ===
    float purpleActive = 0.0;
    constexpr sampler texSampler(mag_filter::linear, min_filter::linear);

    if (hasTexture > 0.5) {
        // Sample the underlying window texture (flip Y for screen capture coords)
        float2 texUV = float2(uv.x, 1.0 - uv.y);
        float3 windowColor = windowTex.sample(texSampler, texUV).rgb;

        // Detect purple at this pixel
        float localPurple = detectPurple(windowColor);

        // Sample 4 neighbors for detection spread (CPU optimized)
        float2 texelSize = float2(1.0 / 1920.0, 1.0 / 1080.0);
        float neighborPurple = 0.0;
        neighborPurple += detectPurple(windowTex.sample(texSampler, texUV + float2(-texelSize.x * 8.0, 0.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, texUV + float2(texelSize.x * 8.0, 0.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, texUV + float2(0.0, -texelSize.y * 8.0)).rgb);
        neighborPurple += detectPurple(windowTex.sample(texSampler, texUV + float2(0.0, texelSize.y * 8.0)).rgb);
        neighborPurple *= 0.25;

        // Combine local and neighbor detection
        purpleActive = max(localPurple, neighborPurple * 0.9);

        // Lower threshold - trigger on near-purple too
        purpleActive = smoothstep(0.1, 0.3, purpleActive);

        // Dampen feedback oscillation — cap self-amplification
        purpleActive = min(purpleActive, 0.7);

        // Sample purple from ABOVE — wobbly organic drip paths
        // Each sample point follows a sine-warped path upward, so the drip
        // meanders like running liquid, not a straight column
        float dripT = time * 0.0003;
        float step1 = texelSize.y * 25.0;
        float purpleAbove = 0.0;
        // 4 samples at increasing distances above, each with unique wobble
        float d1 = step1;
        float d2 = step1 * 3.0;
        float d3 = step1 * 7.0;
        float d4 = step1 * 14.0;
        // Wobble: x offset is sin of (y + time), different freq per sample
        // Creates organic meandering paths that change slowly over time
        float w1 = sin(texUV.y * 15.0 + dripT + 0.0) * texelSize.x * 12.0;
        float w2 = sin(texUV.y * 11.0 + dripT * 1.3 + 2.0) * texelSize.x * 20.0;
        float w3 = sin(texUV.y * 7.0 + dripT * 0.7 + 4.5) * texelSize.x * 30.0;
        float w4 = sin(texUV.y * 5.0 + dripT * 1.1 + 1.2) * texelSize.x * 45.0;
        float pA1 = detectPurple(windowTex.sample(texSampler, float2(texUV.x + w1, max(texUV.y - d1, 0.0))).rgb);
        float pA2 = detectPurple(windowTex.sample(texSampler, float2(texUV.x + w2, max(texUV.y - d2, 0.0))).rgb);
        float pA3 = detectPurple(windowTex.sample(texSampler, float2(texUV.x + w3, max(texUV.y - d3, 0.0))).rgb);
        float pA4 = detectPurple(windowTex.sample(texSampler, float2(texUV.x + w4, max(texUV.y - d4, 0.0))).rgb);
        // Closer samples stronger, far samples fade — drip thins as it falls
        purpleAbove = pA1 * 0.9 + pA2 * 0.65 + pA3 * 0.4 + pA4 * 0.2;
        purpleAbove = smoothstep(0.15, 0.5, purpleAbove) * 0.7;
        // Drip follows ray paths — brighter near rays, invisible far from them
        purpleAbove *= (0.2 + rays * 2.5);
        purpleActive = max(purpleActive, purpleAbove);

        // Apply purple timing
        purpleActive *= purpleTimeScale;
    }

    // === DRIP DOWN EFFECT - flows downward from purple sources ===
    float dripSpeed = 0.015;
    float dripFlow = time * dripSpeed;  // Moves downward (subtract from y)
    float dripY = uv.y - dripFlow;  // Negative = pattern moves down screen

    // Vertical column structure — each x position is a drip channel
    float columnHash = hash(float2(floor(uv.x * 40.0), 0.0));
    float columnActive = step(0.5, columnHash);  // ~50% of columns drip

    // Drip waves biased downward — asymmetric sawtooth-ish shapes
    float dripWave1 = pow(fract(dripY * 8.0 + uv.x * 1.5), 2.0);  // Sharp top, soft tail
    float dripWave2 = pow(fract(dripY * 5.0 + uv.x * 2.5 + 0.3), 2.5);
    float dripWave3 = pow(fract(dripY * 13.0 + uv.x * 0.8 + 0.7), 1.8);

    // Glitch hash for randomized drip breaks
    float glitchHash = hash(float2(floor(uv.x * 50.0), floor(dripY * 30.0)));
    float glitchBreak = step(0.7, glitchHash);

    // Drip strength increases toward bottom (gravity acceleration)
    float gravity = uv.y * uv.y;  // Quadratic — accelerates as it falls
    float dripStrength = gravity * 0.9 * columnActive;

    // Combine waves for organic drip pattern
    float combinedDrip = (dripWave1 * 0.5 + dripWave2 * 0.3 + dripWave3 * 0.2) * dripStrength;
    combinedDrip *= (1.0 - glitchBreak * 0.6);

    // Drip hard near purple
    float dripPurple = purpleActive * combinedDrip * 3.0;
    float dripEcho = purpleActive * sin(dripY * 25.0 + uv.x * 8.0) * 0.8 * dripStrength;
    // Extra heavy drip streaks in active columns
    float dripHeavy = purpleActive * pow(fract(dripY * 3.0 + columnHash), 3.0) * columnActive * 2.0 * gravity;

    float totalDrip = dripPurple + dripEcho + dripHeavy;
    purpleActive = clamp(purpleActive + totalDrip, 0.0, 1.0);

    // === MOIRE SCANLINE EFFECT ===
    // 5 layers at prime frequencies - interference creates moire
    float glitchTime = floor(time * 0.001);

    // Five scanline grids at prime frequencies
    float scan1 = floor(uv.y * 1997.0);
    float scan2 = floor(uv.y * 673.0);
    float scan3 = floor(uv.y * 307.0);
    float diagScan1 = floor((uv.x * 0.6 + uv.y) * 1117.0);
    float diagScan2 = floor((uv.x * -0.4 + uv.y) * 853.0);

    // Single hash per layer
    float r1 = hash(float2(glitchTime, scan1));
    float r2 = hash(float2(glitchTime, scan2));
    float r3 = hash(float2(glitchTime, scan3));
    float rd1 = hash(float2(glitchTime, diagScan1));
    float rd2 = hash(float2(glitchTime, diagScan2));

    // Skip some scanlines randomly
    float present1 = step(0.25, r1);
    float present2 = step(0.20, r2);
    float present3 = step(0.30, r3);
    float presentD1 = step(0.35, rd1);
    float presentD2 = step(0.40, rd2);

    // Glitches concentrated near purple
    float glitchThreshold = mix(0.95, 0.15, purpleActive);

    // Combine layers - moire from interference
    float glitchStrength = step(glitchThreshold, r1) * present1 * 0.25 +
                           step(glitchThreshold, r2) * present2 * 0.25 +
                           step(glitchThreshold, r3) * present3 * 0.20 +
                           step(glitchThreshold, rd1) * presentD1 * 0.20 +
                           step(glitchThreshold, rd2) * presentD2 * 0.15;

    // Purple scanline glitch with gentle pulse (cheap - one sin)
    float glitchPulse = 0.8 + 0.2 * sin(time * 0.0001);
    float3 glitchColor = neonPurple * glitchStrength * (0.1 + purpleActive * 0.5) * glitchPulse;

    // === PALETTE ROTATION WITH INTERPOLATION (from original shader) ===

    // Randomize stripe width using slow-changing hash - narrower bands
    float stripeHash = fract(sin(floor(time * 0.0001) * 12.9898) * 43758.5453);
    float stripeWidthMult = 1.5 + stripeHash * 2.5;  // Higher = narrower bands

    // Wave distortion for organic movement - glacially slow
    float waveDistortion = sin(uv.x * 6.28318 + time * 0.00005) * 0.08 +
                           cos(uv.y * 4.71239 + time * 0.00008) * 0.05;

    // Spatial phase - color varies across screen + time rotation
    float spatialPhase = (uv.x + uv.y * 0.7 + waveDistortion) * stripeWidthMult + time * 0.000003 + 1.5;
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
        float pulsePhase = time * 0.000008;
        float spatialOffset = (uv.x * 2.0 + uv.y * 3.0) * 6.28318;
        float pulse = sin(pulsePhase * 6.28318 + spatialOffset) * 0.5 + 0.5;
        pulse = pulse * pulse * pulse;  // Sharpen
        float transitionDepth = smoothstep(transitionStart, 1.0, fract(continuousIndex));
        rayColor *= (1.0 + pulse * transitionDepth * 2.5);  // 4x stronger pulse
    }

    // Global luminosity pulse (always active, not just during transitions)
    float globalPulse = sin(time * 0.00001 + uv.y * 4.0) * 0.5 + 0.5;
    globalPulse = globalPulse * globalPulse;
    rayColor *= (1.0 + globalPulse * 0.4);

    // Shift toward purple when purple is active - subtle tint
    rayColor = mix(rayColor, neonPurple, purpleActive * 0.3);

    // Purple reactive aura - moderate glow when purple detected
    float auraPhase = time * 0.0001 + uv.y * 2.0;
    float aura = sin(auraPhase) * 0.5 + 0.5;
    aura = aura * aura;
    float3 purpleAura = deepPurple * aura * purpleActive * 1.5;  // Strong aura

    // Base tint - subtle purple
    float3 baseTint = float3(0.06, 0.02, 0.10);

    // Combine - rays should be visible!
    float3 finalColor = baseTint + rays * rayColor * 1.0 + purpleAura + glitchColor;

    // Fade toward top
    finalColor *= 0.4 + (1.0 - uv.y) * 0.6;

    // Alpha - visible but not overwhelming
    float alpha = (0.4 + rays * 0.5 + purpleActive * 0.4) * opacity;

    return float4(finalColor, alpha);
}
