// Underwater + Glitchy (combined) with Vaporwave color cycling
//
// Composition:
// - Apply glitchy distortion / chromatic aberration to the terminal image.
// - Add underwater rays from bottom with vaporwave color palette cycling
//   only on background pixels, so foreground text stays readable.
//
// Sources:
// - glitchy.glsl: modified version of https://www.shadertoy.com/view/wld3WN
// - underwater.glsl: adapted by Alex Sherwin for Ghostty from https://www.shadertoy.com/view/lljGDt

// ---- Cursor-based brightness controls ----
// Requires Ghostty 1.2.0+ for iTimeCursorChange uniform
// Set CURSOR_BRIGHTNESS_ENABLE to 0 if shader fails to compile (older Ghostty)
#define CURSOR_BRIGHTNESS_ENABLE 1        // Set to 0 to disable (or if compile fails)
#define CURSOR_DIM_DURATION 145.0         // Seconds to full brightness (~2.4 minutes)
#define CURSOR_MIN_BRIGHTNESS 0.15        // Minimum brightness right after cursor move
#define CURSOR_BRIGHTNESS_CURVE 0.5       // Easing: <1 = ease-in (slow start), >1 = ease-out

// ---- External ray sweep controls ----
#define EXTERNAL_RAY_ENABLE 1             // Set to 0 to disable
#define EXTERNAL_RAY_PERIOD 20.0          // Seconds for full sweep cycle
#define EXTERNAL_RAY_INTENSITY 0.12       // Intensity of sweep effect
#define EXTERNAL_RAY_WIDTH 0.3            // Width of sweep beam (0-1)

// ---- Underwater controls ----
#define TARGET_COLOR vec3(0.0, 0.0, 0.0) // #000000 pure black
#define COLOR_TOLERANCE 0.15            // tolerance around TARGET_COLOR (increased from 0.03)
#define UNDERWATER_RAY_STRENGTH 0.75  // Strength of rays added to background (was 0.65)
#define FORCE_BACKGROUND_BLACK 1      // 1: force background pixels to pure black
// Higher = fewer rays / blacker baseline
#define UNDERWATER_RAY_CUTOFF 0.85  // Cutoff threshold - higher = thinner rays (was 0.75)
// Ray origin + motion tuning
#define UNDERWATER_SPEED_SCALE 0.00002375 // -50% (was 0.0000475)
#define GRECAS_SPEED_SCALE 0.0000125      // -50% (was 0.000025)
#define SHARED_SPEED_VARIANCE 0.1         // +/- 10% random variance shared by rays and grecas
// Number of ray sources (origin points) - must be > 1 for proper coverage
#define UNDERWATER_RAY_COUNT 2
// Number of rays per source (must be > 1 for proper ray fan effect)
#define RAYS_PER_SOURCE 2  // Reduced from 3 for performance (slightly thinner rays)
// Vaporwave color palette cycling
#define VAPORWAVE_COLOR_COUNT 8  // Includes triple black at start
#define VAPORWAVE_CYCLE_SECONDS 15.0
#define VAPORWAVE_TRANSITION_SMOOTH 0.7
// Ray sources (y < 0 means "below the screen")
#define UNDERWATER_RAY_POS1_NORM vec2(0.55, -0.10)
#define UNDERWATER_RAY_DIR1_NORM normalize(vec2(1.0, -0.06))
#define UNDERWATER_RAY_POS2_NORM vec2(0.45, -0.10)
#define UNDERWATER_RAY_DIR2_NORM normalize(vec2(-1.0, -0.06))

// ---- Glitch controls ----
#define GLITCH_PERIOD_MIN 15.0
#define GLITCH_PERIOD_MAX 45.0
#define GLITCH_WINDOW_SECONDS 7.0
#define GLITCH_ALWAYS_ON 1
#define GLITCH_STRENGTH 0.0378              // +20% (was 0.0315)
#define GLITCH_TIME_SCALE 0.00000015        // -50% (was 0.0000003)
#define GLITCH_SCANLINE_SPEED 0.000125      // -50% (was 0.00025)
#define GLITCH_DISTORTION_SCALE 0.147       // +20% (was 0.1225)
#define GLITCH_RGB_SPLIT_SCALE 0.189        // +20% (was 0.1575)
#define GLITCH_NOISE_SCALE 0.115            // +20% (was 0.096)
#define GLITCH_SCANLINE_SCALE 0.126         // +20% (was 0.105)
// Boost glitch on dark/dim pixels (background), exclude bright text
#define GLITCH_RAY_BOOST 10.0
#define GLITCH_DIM_THRESHOLD 0.35
#define GLITCH_DIM_SOFTNESS 0.10
#define GLITCH_BASELINE_IN_WINDOW 0.0

// ---- Major glitch event controls ----
#define MAJOR_GLITCH_PERIOD 5.0       // Every 5 seconds for testing
#define MAJOR_GLITCH_DURATION 2.0     // 2 second duration
#define MAJOR_GLITCH_STRENGTH 60.0      // CRANKED for testing (was 35.0)
#define MAJOR_GLITCH_PURPLE_BOOST 40.5  // Purple glitch intensity (+50%, was 27.0)
#define STARTUP_GLITCH_DURATION 0.4     // Startup glitch lasts 0.4 seconds (was 1.2)

// ---- Purple smearing controls ----
#define PURPLE_DETECT_THRESHOLD 0.013  // 1.5x more generous
#define PURPLE_SMEAR_DISTANCE 4.0   // Reduced from 16.0 - tighter around letters
#define PURPLE_SMEAR_STRENGTH 0.85
#define DEBUG_PURPLE 0  // Set to 1 to visualize purple detection

// ---- Grey text outline controls ----
#define GREY_OUTLINE_ENABLE 1             // Set to 0 to disable
#define GREY_OUTLINE_THRESHOLD 0.25       // Luminance threshold for "grey" text
#define GREY_OUTLINE_RANGE 0.45           // Max luminance for grey (above = white, skip)
#define GREY_OUTLINE_RADIUS 3.5           // Outline thickness in pixels (was 2.0)
#define GREY_OUTLINE_STRENGTH 0.95        // How dark the outline is (was 0.85)

// ---- Green background suppression ----
#define GREEN_BG_SUPPRESS 1               // Set to 0 to disable
#define GREEN_BG_THRESHOLD 0.08           // How much greener than R/B to trigger
#define GREEN_BG_DARKEN 0.4               // How much to darken green backgrounds (0-1)

// ---- Aztec/Mayan geometry controls ----
// Feature #2: Angular wave motion (stepped sine for feathered serpent effect)
#define AZTEC_ANGULAR_WAVE 0              // DISABLED for evaluation (was 1) - smoother, fewer ops
#define AZTEC_ANGULAR_STEPS 4.0           // Number of quantization steps

// Feature #3: Diamond distance (L1 norm for angular ray shapes)
#define AZTEC_DIAMOND_DISTANCE 1          // Set to 0 to disable
#define AZTEC_DIAMOND_BLEND 0.3           // Blend factor (0=circular, 1=full diamond)

// Feature #4: Grecas overlay (stepped spiral pattern)
#define AZTEC_GRECAS 1                    // Set to 0 to disable
#define AZTEC_GRECAS_INTENSITY 0.072      // Reduced 20% (was 0.09)
#define AZTEC_GRECAS_SCALE 0.058          // Reduced 20% (was 0.072)

// Feature #5: Stepped edges (terraced ray boundaries like Aztec pyramids)
#define AZTEC_STEPPED_EDGES 1             // Set to 0 to disable
#define AZTEC_EDGE_STEPS 5.0              // Number of terrace levels
#define AZTEC_EDGE_SHARPNESS 0.7          // How sharp the steps are (0=smooth, 1=hard)

// Feature #6: 3D ray depth (volumetric illusion)
#define RAY_3D_DEPTH 1                    // Set to 0 to disable
#define RAY_3D_EDGE_GLOW 6.075            // +50% (was 4.05)
#define RAY_3D_CORE_DARKEN 2.43           // +50% (was 1.62)
#define RAY_3D_HIGHLIGHT_OFFSET 0.30      // +50% (was 0.201)

// Feature #7: Subsurface scattering (translucent material simulation)
// Treats ray intensity as surface height, derives normals for diffuse lighting
#define RAY_SUBSURFACE 1                  // Set to 0 to disable
#define RAY_SSS_LIGHT_DIR vec3(0.4, -0.6, 0.7)  // Light direction (normalized internally)
#define RAY_SSS_SCATTER_DEPTH 1.35        // +50% (was 0.9)
#define RAY_SSS_SCATTER_COLOR vec3(1.0, 0.85, 0.7)  // Warm tint from scattering
#define RAY_SSS_DIFFUSE_STRENGTH 2.25     // +50% (was 1.5)
#define RAY_SSS_RIM_STRENGTH 1.35         // +50% (was 0.9)
#define RAY_SSS_NORMAL_SCALE 4.0          // Reduced for stability (was 13.5)

#define SS(a, b, x) (smoothstep(a, b, x) * smoothstep(b, a, x))

// Cursor uniforms (Ghostty 1.2.0+)
// These are provided by Ghostty when available:
// - iCurrentCursor: vec4(x, y, width, height) current cursor position/size
// - iPreviousCursor: vec4(x, y, width, height) previous cursor position/size
// - iTimeCursorChange: float - time when cursor last moved
// If not available, shader will use fallback (full brightness)

// OPTIMIZATION: Global cached values computed once per frame in mainImage()
// These avoid redundant computation in nested functions
float g_cursorFactor;
float g_speedVariance;

// Cursor-based factor: starts low when cursor moves, ramps up over time
// Used for both brightness and speed (same curve)
float computeCursorFactor() {
#if CURSOR_BRIGHTNESS_ENABLE
	// Time since cursor last changed position
	// If iTimeCursorChange not available, this will be 0 and we get full value
	float timeSinceMove = iTime - iTimeCursorChange;

	// Clamp to duration and normalize to 0-1
	float t = clamp(timeSinceMove / CURSOR_DIM_DURATION, 0.0, 1.0);

	// Apply easing curve (quadratic ease-in: slow start, accelerating)
	t = pow(t, 1.0 / CURSOR_BRIGHTNESS_CURVE);

	// Interpolate from min to full
	return mix(CURSOR_MIN_BRIGHTNESS, 1.0, t);
#else
	return 1.0;
#endif
}

// Cached accessors (use these in all functions)
float cursorFactor() { return g_cursorFactor; }
float cursorBrightness() { return g_cursorFactor; }
float cursorSpeedFactor() { return g_cursorFactor; }

// External ray sweep: simulates light source sweeping across from outside terminal
float externalRaySweep(vec2 uv) {
#if EXTERNAL_RAY_ENABLE
	// Sweep position moves across screen over time (0 to 1)
	// Speed scales with cursor stillness
	float sweepPhase = fract(iTime * cursorSpeedFactor() / EXTERNAL_RAY_PERIOD);

	// Sweep travels from left-below to right-above (diagonal)
	// Position along sweep axis
	float sweepAxis = uv.x * 0.7 + (1.0 - uv.y) * 0.3;  // Mostly horizontal, slight vertical

	// Distance from sweep center
	float distFromSweep = abs(sweepAxis - sweepPhase);

	// Wrap distance for seamless looping
	distFromSweep = min(distFromSweep, 1.0 - distFromSweep);

	// Soft falloff from sweep center
	float sweepIntensity = 1.0 - smoothstep(0.0, EXTERNAL_RAY_WIDTH, distFromSweep);

	// Add slight vertical gradient (brighter at top as if light from above)
	float verticalGrad = 0.7 + uv.y * 0.3;

	return sweepIntensity * verticalGrad * EXTERNAL_RAY_INTENSITY;
#else
	return 0.0;
#endif
}

// Forward declarations for ray computation
float underwaterRayMask(vec2 fragCoord);
// Cached ray mask passed through to avoid redundant computation
// neighborRays: vec4(left, right, up, down) for gradient estimation

#define UI0 1597334673U
#define UI1 3812015801U
#define UI2 uvec2(UI0, UI1)
#define UI3 uvec3(UI0, UI1, 2798796415U)
#define UIF (1. / float(0xffffffffU))

// OPTIMIZED: RGB to HSL conversion (branchless)
vec3 rgb2hsl(vec3 rgb)
{
	float maxC = max(max(rgb.r, rgb.g), rgb.b);
	float minC = min(min(rgb.r, rgb.g), rgb.b);
	float delta = maxC - minC;
	float l = (maxC + minC) * 0.5;

	// Branchless saturation
	float denom = mix(maxC + minC, 2.0 - maxC - minC, step(0.5, l));
	float s = delta / max(denom, 0.0001);
	s *= step(0.0001, delta);  // Zero if no delta

	// Branchless hue calculation
	float isR = step(maxC, rgb.r + 0.0001) * step(rgb.r, maxC + 0.0001);
	float isG = step(maxC, rgb.g + 0.0001) * step(rgb.g, maxC + 0.0001) * (1.0 - isR);
	float isB = 1.0 - isR - isG;

	float invDelta = 1.0 / max(delta, 0.0001);
	float hR = (rgb.g - rgb.b) * invDelta;
	float hG = 2.0 + (rgb.b - rgb.r) * invDelta;
	float hB = 4.0 + (rgb.r - rgb.g) * invDelta;

	float h = (hR * isR + hG * isG + hB * isB) / 6.0;
	h = fract(h);  // Wraps negative to positive

	return vec3(h, s, l);
}

// OPTIMIZED: HSL to RGB conversion (branchless using smoothstep sectors)
vec3 hsl2rgb(vec3 hsl)
{
	float h = hsl.x;
	float s = hsl.y;
	float l = hsl.z;

	float c = (1.0 - abs(2.0 * l - 1.0)) * s;
	float m = l - c * 0.5;

	// Use continuous function: RGB = |H*6 - vec3(0,2,4)| clamped and scaled
	vec3 rgb = clamp(abs(mod(h * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
	return rgb * c + m;
}

// OPTIMIZED: Vaporwave-aware color interpolation (branchless)
vec3 mixVaporwave(vec3 color1, vec3 color2, float t)
{
	vec3 hsl1 = rgb2hsl(color1);
	vec3 hsl2 = rgb2hsl(color2);

	// Choose shorter hue rotation path on the color wheel (branchless)
	float hueDiff = hsl2.x - hsl1.x;
	hueDiff += mix(0.0, -1.0, step(0.5, hueDiff));
	hueDiff += mix(0.0, 1.0, step(hueDiff, -0.5));

	vec3 hslMix = vec3(
		fract(hsl1.x + hueDiff * t),  // fract handles wrap
		mix(hsl1.y, hsl2.y, t),
		mix(hsl1.z, hsl2.z, t)
	);

	return hsl2rgb(hslMix);
}

// OPTIMIZED: Vaporwave color palette as array (single indexed lookup vs 8 branches)
const vec3 VAPORWAVE_PALETTE[8] = vec3[8](
	vec3(1.00, 0.00, 0.973),   // 0: #ff00f8 Hot Magenta
	vec3(0.984, 0.718, 0.145), // 1: #fbb725 Warm Gold
	vec3(0.361, 0.925, 1.00),  // 2: #5cecff Electric Cyan
	vec3(1.00, 0.00, 0.973),   // 3: #ff00f8 Hot Magenta
	vec3(0.361, 0.925, 1.00),  // 4: #5cecff Electric Cyan
	vec3(0.361, 0.925, 1.00),  // 5: #5cecff Electric Cyan
	vec3(0.753, 0.502, 0.816), // 6: #c080d0 Deep Purple
	vec3(0.670, 0.376, 0.929)  // 7: #ab60ed Neon Purple
);

vec3 getVaporwaveColor(int index)
{
	return VAPORWAVE_PALETTE[index];
}

// Hash by David_Hoskins
vec3 hash33(vec3 p)
{
	uvec3 q = uvec3(ivec3(p)) * UI3;
	q = (q.x ^ q.y ^ q.z) * UI3;
	return -1. + 2. * vec3(q) * UIF;
}

// Gradient noise by iq
float gnoise(vec3 x)
{
	vec3 p = floor(x);
	vec3 w = fract(x);
	vec3 u = w * w * w * (w * (w * 6. - 15.) + 10.);
	
	vec3 ga = hash33(p + vec3(0., 0., 0.));
	vec3 gb = hash33(p + vec3(1., 0., 0.));
	vec3 gc = hash33(p + vec3(0., 1., 0.));
	vec3 gd = hash33(p + vec3(1., 1., 0.));
	vec3 ge = hash33(p + vec3(0., 0., 1.));
	vec3 gf = hash33(p + vec3(1., 0., 1.));
	vec3 gg = hash33(p + vec3(0., 1., 1.));
	vec3 gh = hash33(p + vec3(1., 1., 1.));
	
	float va = dot(ga, w - vec3(0., 0., 0.));
	float vb = dot(gb, w - vec3(1., 0., 0.));
	float vc = dot(gc, w - vec3(0., 1., 0.));
	float vd = dot(gd, w - vec3(1., 1., 0.));
	float ve = dot(ge, w - vec3(0., 0., 1.));
	float vf = dot(gf, w - vec3(1., 0., 1.));
	float vg = dot(gg, w - vec3(0., 1., 1.));
	float vh = dot(gh, w - vec3(1., 1., 1.));
	
	float gNoise =
		va + u.x * (vb - va) +
		u.y * (vc - va) +
		u.z * (ve - va) +
		u.x * u.y * (va - vb - vc + vd) +
		u.y * u.z * (va - vc - ve + vg) +
		u.z * u.x * (va - vb - ve + vf) +
		u.x * u.y * u.z * (-va + vb + vc - vd + ve - vf - vg + vh);
	
	return 2. * gNoise;
}

float hash21(vec2 p)
{
	p = fract(p * vec2(233.34, 851.73));
	p += dot(p, p + 23.45);
	return fract(p.x * p.y);
}

// Shared speed variance for rays and grecas (+/- SHARED_SPEED_VARIANCE)
// Changes very slowly over time, same value for entire frame
// OPTIMIZATION: Computed once per frame in mainImage, cached in g_speedVariance
float computeSpeedVariance()
{
	// Very slowly varying: changes every ~60 seconds with smooth interpolation
	float timeSlot = iTime * 0.0167;  // 1/60 = one cycle per minute
	float slot0 = floor(timeSlot);
	float slot1 = slot0 + 1.0;
	// Smooth blend over middle 50% of interval (holds at start/end)
	float rawBlend = fract(timeSlot);
	float blend = smoothstep(0.25, 0.75, rawBlend);

	// Use different prime multipliers for better distribution
	float rand0 = fract(sin(slot0 * 78.233 + 12.9898) * 43758.5453);
	float rand1 = fract(sin(slot1 * 78.233 + 12.9898) * 43758.5453);
	float randVal = mix(rand0, rand1, blend);

	// Clamp randVal to ensure it's truly 0-1, then map to exact +/- range
	randVal = clamp(randVal, 0.0, 1.0);
	return 1.0 + (randVal - 0.5) * 2.0 * SHARED_SPEED_VARIANCE;
}

// Cached accessor
float sharedSpeedVariance() { return g_speedVariance; }

// ============================================================================
// Feature #2: Angular wave motion (Aztec feathered serpent effect)
// Quantizes sine wave to create stepped, angular undulation
// ============================================================================
#if AZTEC_ANGULAR_WAVE
float angularSin(float x)
{
	return floor(sin(x) * AZTEC_ANGULAR_STEPS + 0.5) / AZTEC_ANGULAR_STEPS;
}

float angularCos(float x)
{
	return floor(cos(x) * AZTEC_ANGULAR_STEPS + 0.5) / AZTEC_ANGULAR_STEPS;
}
#endif

// ============================================================================
// Feature #3: Diamond distance (L1/Manhattan norm for angular shapes)
// Creates diamond-shaped falloff instead of circular
// ============================================================================
#if AZTEC_DIAMOND_DISTANCE
float diamondLength(vec2 v)
{
	float circular = length(v);
	float diamond = abs(v.x) + abs(v.y);
	return mix(circular, diamond * 0.707, AZTEC_DIAMOND_BLEND); // 0.707 normalizes L1 to ~L2 scale
}
#endif

// ============================================================================
// Feature #4: Grecas overlay (Aztec stepped spiral/fret pattern)
// Adds faint angular spiral geometry to ray regions
// ============================================================================
#if AZTEC_GRECAS
float grecasPattern(vec2 uv, float rayIntensity)
{
	// BRANCHLESS: use smoothstep mask instead of early return
	float activeMask = smoothstep(0.0, 0.02, rayIntensity);

	// Diamond distance from center for angular feel
	vec2 centered = uv - 0.5;
	float dist = abs(centered.x) + abs(centered.y);

	// Angular position (quantized to create stepped look)
	float angle = atan(centered.y, centered.x);
	float quantizedAngle = floor(angle * 4.0 / 3.14159) / 4.0 * 3.14159;

	// Stepped spiral: angle + distance creates spiral, floor() creates steps
	// Speed scales with cursor stillness
	float spiral = quantizedAngle + dist * AZTEC_GRECAS_SCALE * 20.0 + iTime * cursorSpeedFactor() * GRECAS_SPEED_SCALE * sharedSpeedVariance();

	// Create thin lines at step boundaries (branchless)
	float pattern = fract(spiral * 3.0);
	float line = step(0.92, pattern) + step(pattern, 0.08);

	return line * AZTEC_GRECAS_INTENSITY * rayIntensity * activeMask;
}
#endif

// ============================================================================
// Feature #5: Stepped edges (Aztec pyramid terrace effect)
// Quantizes ray intensity to create bold, geometric stepped boundaries
// ============================================================================
#if AZTEC_STEPPED_EDGES
float aztecSteppedEdge(float rayValue)
{
	// Quantize to discrete steps
	float stepped = floor(rayValue * AZTEC_EDGE_STEPS) / AZTEC_EDGE_STEPS;
	// Blend between smooth and stepped based on sharpness
	return mix(rayValue, stepped, AZTEC_EDGE_SHARPNESS);
}
#endif

// ============================================================================
// Feature #6: 3D ray depth (volumetric light beam illusion)
// Creates the appearance of cylindrical light beams with edge highlights
// OPTIMIZED: Uses pre-computed neighbor rays instead of resampling
// ============================================================================
#if RAY_3D_DEPTH
vec3 apply3DRayDepth(vec3 rayColor, float rayIntensity, vec4 neighborRays)
{
	// BRANCHLESS: use smoothstep mask instead of early return
	float activeMask = smoothstep(0.0, 0.02, rayIntensity);

	// neighborRays = vec4(left, right, up, down)
	// Use diagonal approximation: L+U vs R+D for edge detection
	float rayLU = (neighborRays.x + neighborRays.z) * 0.5;
	float rayRD = (neighborRays.y + neighborRays.w) * 0.5;

	// Edge detection: high difference = we're at an edge
	float edgeness = abs(rayLU - rayRD);

	// Highlight on the "upper" edge (light source side)
	float highlight = smoothstep(0.0, 0.3, rayRD - rayLU + RAY_3D_HIGHLIGHT_OFFSET);

	// Core detection: if surrounded by ray, we're in the middle
	float minNeighbor = min(min(neighborRays.x, neighborRays.y), min(neighborRays.z, neighborRays.w));
	float coreness = minNeighbor * rayIntensity;

	// Apply 3D shading (branchless via mix)
	vec3 result = rayColor;
	result += rayColor * edgeness * RAY_3D_EDGE_GLOW;
	result += vec3(1.0, 0.95, 0.9) * highlight * edgeness * 0.3;
	result *= 1.0 - coreness * RAY_3D_CORE_DARKEN;

	// Blend between original and effect based on activation
	return mix(rayColor, result, activeMask);
}
#endif

// ============================================================================
// Feature #7: Subsurface scattering (translucent material simulation)
// Treats ray intensity as a height field, derives normals for lighting
// Simulates light diffusing through a waxy/translucent material
// OPTIMIZED: Uses pre-computed neighbor rays instead of resampling
// ============================================================================
#if RAY_SUBSURFACE
vec3 applySubsurfaceScattering(vec3 rayColor, float rayIntensity, vec4 neighborRays)
{
	// BRANCHLESS: use smoothstep mask instead of early return
	float activeMask = smoothstep(0.0, 0.02, rayIntensity);

	// neighborRays = vec4(left, right, up, down)
	// Compute gradient from pre-sampled neighbors
	float dX = (neighborRays.y - neighborRays.x) * RAY_SSS_NORMAL_SCALE;
	float dY = (neighborRays.z - neighborRays.w) * RAY_SSS_NORMAL_SCALE;

	// Derive normal from gradient: N = normalize(-dX, -dY, 1)
	vec3 normal = normalize(vec3(-dX, -dY, 1.0));

	// Light direction (from above-right, into the screen)
	vec3 lightDir = normalize(RAY_SSS_LIGHT_DIR);

	// Basic diffuse lighting (N dot L)
	float NdotL = max(dot(normal, lightDir), 0.0);
	float diffuse = NdotL * RAY_SSS_DIFFUSE_STRENGTH;

	// Rim lighting: light wrapping around edges (simulates translucency)
	float NdotL_back = max(dot(normal, -lightDir), 0.0);
	float rim = pow(1.0 - abs(dot(normal, vec3(0.0, 0.0, 1.0))), 2.0);
	float rimLight = rim * RAY_SSS_RIM_STRENGTH;

	// Subsurface scatter: light penetrates and exits with color shift
	float scatterAmount = rayIntensity * RAY_SSS_SCATTER_DEPTH;
	vec3 scatteredLight = RAY_SSS_SCATTER_COLOR * scatterAmount * (0.5 + NdotL_back * 0.5);

	// Combine lighting
	vec3 result = rayColor;
	result *= (0.7 + diffuse * 0.6);
	result += rayColor * rimLight;
	result += scatteredLight * rayIntensity;

	// Blend between original and effect based on activation
	return mix(rayColor, result, activeMask);
}
#endif

// ============================================================================
// Grey text outline (black border around grey/dim text for readability)
// OPTIMIZED: Accepts pre-computed center luminance to avoid redundant texture sample
// ============================================================================
#if GREY_OUTLINE_ENABLE
float greyTextOutline(vec2 uv, vec2 pixelSize, float centerLum)
{
	// BRANCHLESS: use smoothstep mask instead of early return
	float darkMask = 1.0 - smoothstep(0.1, 0.2, centerLum);

	// Always sample 4 neighbors (branchless - multiply by mask at end)
	vec2 offset = pixelSize * GREY_OUTLINE_RADIUS;
	float lumR = dot(texture(iChannel0, uv + vec2(offset.x, 0.0)).rgb, vec3(0.299, 0.587, 0.114));
	float lumL = dot(texture(iChannel0, uv - vec2(offset.x, 0.0)).rgb, vec3(0.299, 0.587, 0.114));
	float lumU = dot(texture(iChannel0, uv + vec2(0.0, offset.y)).rgb, vec3(0.299, 0.587, 0.114));
	float lumD = dot(texture(iChannel0, uv - vec2(0.0, offset.y)).rgb, vec3(0.299, 0.587, 0.114));

	// Check if any neighbor is grey (between threshold and range) - branchless
	float isNearGrey = 0.0;
	isNearGrey = max(isNearGrey, step(GREY_OUTLINE_THRESHOLD, lumR) * step(lumR, GREY_OUTLINE_RANGE));
	isNearGrey = max(isNearGrey, step(GREY_OUTLINE_THRESHOLD, lumL) * step(lumL, GREY_OUTLINE_RANGE));
	isNearGrey = max(isNearGrey, step(GREY_OUTLINE_THRESHOLD, lumU) * step(lumU, GREY_OUTLINE_RANGE));
	isNearGrey = max(isNearGrey, step(GREY_OUTLINE_THRESHOLD, lumD) * step(lumD, GREY_OUTLINE_RANGE));

	// Mask multiplication makes result 0 when darkMask is 0 (branchless early-out)
	return isNearGrey * GREY_OUTLINE_STRENGTH * darkMask;
}
#endif

float glitchPeriod(float timeSec)
{
	float bucket = floor(timeSec / 60.0);
	float r = hash21(vec2(bucket, bucket + 91.73));
	return mix(GLITCH_PERIOD_MIN, GLITCH_PERIOD_MAX, r);
}

float glitchGate(float timeSec)
{
	float period = glitchPeriod(timeSec);
	float win = min(GLITCH_WINDOW_SECONDS, period * 0.5);
	float cycle = floor(timeSec / period);
	float offsetRand = hash21(vec2(cycle, cycle + 23.17));
	float offset = offsetRand * (period - win);
	float phase = mod(timeSec, period);
	return SS(offset, offset + win, phase);
}

float rayStrength(vec2 raySource, vec2 rayRefDirection, vec2 coord, float seedA, float seedB, float speed)
{
	vec2 sourceToCoord = coord - raySource;

	// Cursor-adjusted time for animation (slows when typing, speeds when reading)
	float adjustedTime = iTime * cursorSpeedFactor();

	// Feature #3: Diamond distance - use L1 norm for angular ray shapes
#if AZTEC_DIAMOND_DISTANCE
	float distance = diamondLength(sourceToCoord);
#else
	float distance = length(sourceToCoord);
#endif
	float maxDistance = iResolution.y * 1.5;
	float distanceRatio = clamp(distance / maxDistance, 0.0, 1.0);

	// Create perpendicular offset that varies with distance (makes rays curve)
	vec2 perpendicular = vec2(-sourceToCoord.y, sourceToCoord.x);
	perpendicular = normalize(perpendicular);

	// Feature #2: Angular wave motion - stepped sine for feathered serpent effect
#if AZTEC_ANGULAR_WAVE
	float bendAmount =
		angularSin(distance * 0.009 * seedA + adjustedTime * speed * 0.5) * 200.0 +
		angularCos(distance * 0.015 * seedB + adjustedTime * speed * 0.3) * 125.0 +
		angularSin(distance * 0.024 * (seedA + seedB) + adjustedTime * speed * 0.7) * 75.0;
#else
	// OPTIMIZED: Simplified trig (3 calls instead of 8) - not algebraically equivalent but visually similar
	// OLD CODE (for reversion):
	// float bendAmount =
	//     sin(distance * 0.009 * seedA + adjustedTime * speed * 0.5) * 200.0 +
	//     cos(distance * 0.015 * seedB + adjustedTime * speed * 0.3) * 125.0 +
	//     sin(distance * 0.024 * (seedA + seedB) + adjustedTime * speed * 0.7) * 75.0;
	float combinedFreq = distance * 0.012 * (seedA + seedB) * 0.5 + adjustedTime * speed * 0.5;
	float bendAmount = sin(combinedFreq) * 280.0 + cos(combinedFreq * 1.7) * 120.0;
#endif

	bendAmount *= distanceRatio * distanceRatio;

	vec2 bentCoord = coord + perpendicular * bendAmount;
	vec2 bentSourceToCoord = bentCoord - raySource;

	float cosAngle = dot(normalize(bentSourceToCoord), rayRefDirection);
	float dither = hash21(coord) * 0.015 - 0.0075;

	float bendFactor = 1.0 + distanceRatio * 2.0;

	// Feature #2: Angular wave motion in width variation
#if AZTEC_ANGULAR_WAVE
	float widthVariation =
		0.5 * angularSin(cosAngle * seedA * 0.5 * bendFactor + adjustedTime * speed * 0.8) +
		0.3 * angularCos(cosAngle * seedB * 1.5 * bendFactor + adjustedTime * speed * 1.2) +
		0.2 * angularSin(cosAngle * (seedA + seedB) * 2.0 * bendFactor + adjustedTime * speed * 0.6);

	float ray = clamp(
		(0.65 + 0.35 * angularSin(cosAngle * seedA * bendFactor + adjustedTime * speed)) +
			(0.5 + 0.4 * angularCos(-cosAngle * seedB * bendFactor + adjustedTime * speed)) +
			widthVariation + dither,
		0.0, 1.0);
#else
	// OPTIMIZED: Simplified width + ray calc (1 trig instead of 5)
	// OLD CODE (for reversion):
	// float widthVariation =
	//     0.5 * sin(cosAngle * seedA * 0.5 * bendFactor + adjustedTime * speed * 0.8) +
	//     0.3 * cos(cosAngle * seedB * 1.5 * bendFactor + adjustedTime * speed * 1.2) +
	//     0.2 * sin(cosAngle * (seedA + seedB) * 2.0 * bendFactor + adjustedTime * speed * 0.6);
	// float ray = clamp(
	//     (0.65 + 0.35 * sin(cosAngle * seedA * bendFactor + adjustedTime * speed)) +
	//         (0.5 + 0.4 * cos(-cosAngle * seedB * bendFactor + adjustedTime * speed)) +
	//         widthVariation + dither,
	//     0.0, 1.0);
	float rayPhase = cosAngle * (seedA + seedB) * 0.5 * bendFactor + adjustedTime * speed;
	float ray = clamp(1.15 + 0.75 * sin(rayPhase) + dither, 0.0, 1.0);
#endif

	ray = clamp((ray - UNDERWATER_RAY_CUTOFF) / max(1.0 - UNDERWATER_RAY_CUTOFF, 1e-6), 0.0, 1.0);

	// Distance-based falloff
	float distanceFade = 1.0 - smoothstep(0.0, maxDistance, distance);

	return ray * distanceFade;
}

float dimPixelMask(vec3 rgb)
{
	float luminance = dot(rgb, vec3(0.299, 0.587, 0.114));
	return 1.0 - smoothstep(GLITCH_DIM_THRESHOLD, GLITCH_DIM_THRESHOLD + GLITCH_DIM_SOFTNESS, luminance);
}

// Startup fade-in for rays (branchless)
float rayStartupFade()
{
	float startupProgress = clamp(iTime / 180.0, 0.0, 1.0);
	float fade = startupProgress * startupProgress * startupProgress * startupProgress * startupProgress;
	return mix(fade, 1.0, step(180.0, iTime));
}

vec3 glitchyColor(vec2 uv, vec2 fragCoord, float boostMask, float nearPurple, float protectedMask, float textureVariance)
{
	float t = iTime * GLITCH_TIME_SCALE;
	vec3 passthrough = texture(iChannel0, uv).rgb;

	// Compute major glitch activation (periodic intense glitch events)
	float majorGlitchCycle = mod(iTime, MAJOR_GLITCH_PERIOD);
	float majorGlitchActive = smoothstep(0.0, 0.1, majorGlitchCycle) *
	                          (1.0 - smoothstep(MAJOR_GLITCH_DURATION - 0.1, MAJOR_GLITCH_DURATION, majorGlitchCycle));
	// Also activate during startup
	majorGlitchActive = max(majorGlitchActive, 1.0 - smoothstep(0.0, STARTUP_GLITCH_DURATION, iTime));

	// For highly protected pixels (bright white), return passthrough immediately
	if (protectedMask > 0.7) return passthrough;

	float localMask = clamp(boostMask, 0.0, 1.0);
	// Reduce glitch on protected pixels (white/red)
	localMask *= (1.0 - protectedMask * 0.99);
	
	// Reduce glitch near heterogeneous areas (images, dense text)
	// BUT don't reduce near purple - purple text should still glitch
	float textureProtection = smoothstep(0.005, 0.03, textureVariance);
	textureProtection *= (1.0 - nearPurple);
	localMask *= (1.0 - textureProtection * 0.9);
	
	// Near purple: ensure minimum glitch even without rays
	float purpleMinGlitch = nearPurple * 1.125;  // +50% (was 0.75)
	localMask = max(localMask, purpleMinGlitch);
	
	if (localMask <= 0.0) return passthrough;

	float gate = 1.0;
#if !GLITCH_ALWAYS_ON
	gate = glitchGate(iTime);
#endif

	float dimMask = dimPixelMask(passthrough);
	
	float glitchAmount = clamp(gate * GLITCH_STRENGTH * (localMask * dimMask * GLITCH_RAY_BOOST), 0.0, 3.0);
	
	// Boost glitch near purple text
	float purpleGlitchMult = 1.0 + nearPurple * 18.0;  // +50% (was 12.0)
	glitchAmount *= purpleGlitchMult;
	
	if (glitchAmount <= 0.0) return passthrough;

	vec3 col = vec3(0.);
	float effectMask = localMask * dimMask;
	
	// Extra intensity near purple
	float purpleIntensityMult = 1.0 + nearPurple * 13.5;  // +50% (was 9.0)
	
	float rgbSplit = GLITCH_RGB_SPLIT_SCALE * mix(0.35, 4.0, effectMask) * purpleIntensityMult;
	float distScale = GLITCH_DISTORTION_SCALE * mix(0.35, 5.0, effectMask) * purpleIntensityMult;
	float noiseScale = GLITCH_NOISE_SCALE * mix(0.25, 3.5, effectMask) * purpleIntensityMult;
	float scanScale = GLITCH_SCANLINE_SCALE * mix(0.25, 3.5, effectMask) * purpleIntensityMult;

	vec2 eps = vec2((2.0 + majorGlitchActive * 20.0 + nearPurple * 10.0) / iResolution.x * rgbSplit, 0.);
	vec2 st = vec2(0.);

	// analog distortion - slow variation only (no subsecond flicker)
	// Speed scales with cursor stillness
	float y = uv.y * iResolution.y;
	float slowT = iTime * cursorSpeedFactor() * 0.1;  // Cursor-adjusted, 10x slower than realtime
	float distortion = sin(y * 0.01 + slowT * 0.5) * sin(y * 0.02 + slowT * 0.25) * (glitchAmount * 4. + majorGlitchActive * 0.8) * distScale;
	
	st = uv + vec2(distortion, 0.);

	// chromatic aberration - branchless (always sample, blend with mask)
	vec3 chromaCol;
	chromaCol.r = textureLod(iChannel0, st + eps + distortion, 0.).r;
	chromaCol.g = textureLod(iChannel0, st, 0.).g;
	chromaCol.b = textureLod(iChannel0, st - eps - distortion, 0.).b;
	vec3 plainCol = textureLod(iChannel0, uv, 0.).rgb;
	col = mix(plainCol, chromaCol, step(0.01, glitchAmount));
	
	// Suppress green channel aggressively
	col.g *= mix(0.7, 0.05, effectMask);

	// white noise - only when there's glitch activity (slow variation, cursor-scaled)
	if (glitchAmount > 0.01) {
		float slowNoiseSeed = floor(iTime * cursorSpeedFactor());  // Changes once per second, cursor-scaled
		float noise = fract(sin(dot(fragCoord + slowNoiseSeed, vec2(12.9898, 78.233))) * 43758.5453);
		col += (.15 + .65 * glitchAmount) * noise * 0.2 * noiseScale;
	}
	
	// Scanlines - always visible, dramatically boosted near glitch and purple
	// Use slow time for scanline scroll (no subsecond variation), cursor-adjusted
	float scanlineTime = floor(iTime * cursorSpeedFactor() * 2.0) * 0.5;  // Steps every 0.5s, cursor-scaled
	float scanlineY = uv.y * iResolution.y + scanlineTime * GLITCH_SCANLINE_SPEED * 1000.0;
	float heightVar = 0.005 + fract(sin(scanlineY * 0.001) * 43758.5453) * 49.995; // 0.005 to 50.0 (widened)
	float scanlinePattern = sin(scanlineY * heightVar);
	float scanlineThreshold = 0.1; // 90% coverage (more visible)
	float showScanline = step(scanlineThreshold, fract(scanlineY * 0.01));

	// Base intensity much more visible, boosted near purple and glitch
	float baseIntensity = 0.15; // 5x more visible base (was 0.03)
	float purpleBoost = nearPurple * 13.5; // Strong purple boost (+50%, was 9.0)
	float glitchBoost = 1.0 + glitchAmount * 12.0; // 12x dramatic boost near glitch
	float scanlineIntensity = (baseIntensity + purpleBoost) * glitchBoost;

	// Always apply scanlines (independent from glitch condition)
	col -= scanlineIntensity * abs(scanlinePattern) * showScanline * scanScale;

	vec3 glitched = mix(passthrough, col, clamp(glitchAmount, 0.0, 1.0));
	
	// Darken glitch based on original pixel brightness (not on protected text)
	float originalLuminance = dot(passthrough, vec3(0.299, 0.587, 0.114));
	float darkening = 1.0 - originalLuminance;
	float darkeningAmount = darkening * glitchAmount * (1.0 - protectedMask);
	glitched *= mix(1.0, 0.1, darkeningAmount);
	
	return clamp(glitched, 0.0, 1.0);
}

float underwaterRayMask(vec2 fragCoord)
{
	vec2 coord = fragCoord;
	float r = 0.0;

	// Speed scales with cursor stillness (compute once for efficiency)
	float speedFactor = cursorSpeedFactor();

	// Process each ray source
	for (int source = 0; source < UNDERWATER_RAY_COUNT; source++) {
		// Animate ray origin - non-repeating motion using incommensurate frequencies
		float t1 = iTime * speedFactor * 0.003 * (1.0 + float(source) * 0.3);

		float xOffset1 = (sin(t1) + sin(t1 * 1.618) * 0.7) * iResolution.x * 0.2;
		float yOffset1 = (sin(t1 * 0.7071) + cos(t1 * 1.2247) * 0.5) * iResolution.y * 0.06;

		vec2 rayPosBase;
		vec2 baseDir;
		if (source == 0) {
			rayPosBase = UNDERWATER_RAY_POS1_NORM * iResolution.xy;
			baseDir = UNDERWATER_RAY_DIR1_NORM;
		} else {
			rayPosBase = UNDERWATER_RAY_POS2_NORM * iResolution.xy;
			baseDir = UNDERWATER_RAY_DIR2_NORM;
		}

		vec2 rayPos = rayPosBase + vec2(xOffset1, yOffset1);

		// Add rotation animation to the entire ray fan
		float rotationSpeed = 0.000006;
		float rotationAngle = iTime * speedFactor * rotationSpeed * (source == 0 ? 1.0 : -1.0);
		
		// Create multiple rays emanating from this source at different angles
		for (int i = 0; i < RAYS_PER_SOURCE; i++) {
			float angleOffset = (float(i) / float(RAYS_PER_SOURCE)) * 3.14159 * 0.4 - 0.2 * 3.14159;
			float totalAngle = angleOffset + rotationAngle;
			float cosA = cos(totalAngle);
			float sinA = sin(totalAngle);
			
			vec2 rayDir = vec2(
				baseDir.x * cosA - baseDir.y * sinA,
				baseDir.x * sinA + baseDir.y * cosA
			);
			
			float seedA = 36.2214 + float(i) * 13.7 + float(source) * 100.0;
			float seedB = 21.11349 + float(i) * 7.3 + float(source) * 100.0;
			float speed = (1.0 + float(i) * 0.1) * UNDERWATER_SPEED_SCALE * sharedSpeedVariance();
			
			float weight = mix(0.8, 1.0, 1.0 - abs(float(i) - float(RAYS_PER_SOURCE - 1) * 0.5) / float(RAYS_PER_SOURCE));
			r += weight * rayStrength(rayPos, rayDir, coord, seedA, seedB, speed);
		}
	}
	
	r /= float(RAYS_PER_SOURCE * UNDERWATER_RAY_COUNT);

	// Feature #5: Apply Aztec stepped edges for terraced pyramid look
#if AZTEC_STEPPED_EDGES
	r = aztecSteppedEdge(r);
#endif

	return clamp(r, 0.0, 1.0);
}

// OPTIMIZED: accepts pre-computed center ray to avoid duplicate underwaterRayMask call
// Uses screen-space derivatives (dFdx/dFdy) instead of 4 extra underwaterRayMask calls
vec3 underwaterRays(vec2 fragCoord, float r)
{
	// OPTIMIZATION: Use GPU screen-space derivatives to estimate neighbors
	// dFdx/dFdy give us the rate of change of r across adjacent pixels
	// This is essentially free on GPUs vs recomputing underwaterRayMask 4 times
	float drdx = dFdx(r);  // Change in r per pixel in X
	float drdy = dFdy(r);  // Change in r per pixel in Y

	// Reconstruct approximate neighbor values from derivatives
	// Scale by 2.0 to match the original 2-pixel offset sampling
	float rayLeft  = r - drdx * 2.0;
	float rayRight = r + drdx * 2.0;
	float rayUp    = r + drdy * 2.0;
	float rayDown  = r - drdy * 2.0;

	// Clamp to valid range (derivatives can extrapolate outside 0-1)
	vec4 neighborRays = clamp(vec4(rayLeft, rayRight, rayUp, rayDown), 0.0, 1.0);

	// Spatial color cycling - color varies across the screen with randomized stripe widths
	vec2 uv = fragCoord / iResolution.xy;
	
	// Randomize stripe width per color using hash - much wider stripes
	float hash = fract(sin(floor(iTime * 0.001) * 12.9898) * 43758.5453); // 10x slower
	float stripeWidthMult = 0.5 + hash * 1.5; // Lower multiplier = wider stripes (0.5x to 2.0x, was 2.0-5.0)
	
	// Add subtle organic wave distortion to the spatial phase
	float waveDistortion = sin(uv.x * 3.14159 * 2.0 + iTime * 0.014) * 0.08 +   // -30% (was 0.02)
	                       cos(uv.y * 3.14159 * 1.5 + iTime * 0.021) * 0.05;    // -30% (was 0.03)

	float spatialPhase = (uv.x + uv.y * 0.7 + waveDistortion) * 1.0 * stripeWidthMult + iTime * 0.0014; // -30% (was 0.002)
	float continuousIndex = mod(spatialPhase, float(VAPORWAVE_COLOR_COUNT));
	int colorIndex = int(floor(continuousIndex));
	float colorBlend = fract(continuousIndex);
	
	// Apply easing: hold on pure colors, transition in random small intervals
	float transitionHash = fract(sin(float(colorIndex) * 12.9898) * 43758.5453);
	float transitionStart = 0.88 + transitionHash * 0.08; // Random start between 0.88 and 0.96
	float isTransitioning = 0.0;
	if (colorBlend < transitionStart) {
		colorBlend = 0.0;
	} else {
		isTransitioning = 1.0;
		float normalizedTransition = (colorBlend - transitionStart) / (1.0 - transitionStart);
		colorBlend = normalizedTransition * normalizedTransition * (3.0 - 2.0 * normalizedTransition);
	}
	
	vec3 color1 = getVaporwaveColor(colorIndex % VAPORWAVE_COLOR_COUNT);
	vec3 color2 = getVaporwaveColor((colorIndex + 1) % VAPORWAVE_COLOR_COUNT);
	// RGB-only blend (faster than HSL mixVaporwave, subtle hue difference)
	vec3 currentColor = mix(color1, color2, colorBlend);
	
	// Add periodic pulses during color transitions
	if (isTransitioning > 0.5) {
		// Create pulses that travel along the transition zones
		float pulseFrequency = 0.028; // -50% (was 0.056) - one pulse every ~36 seconds
		float pulsePhase = iTime * pulseFrequency;
		
		// Create multiple pulse waves at different positions in the transition zone
		float spatialOffset = (uv.x * 2.0 + uv.y * 3.0) * 6.28318; // Different phase per position
		float pulse1 = sin(pulsePhase * 6.28318 + spatialOffset) * 0.5 + 0.5;
		float pulse2 = sin(pulsePhase * 6.28318 * 1.618 + spatialOffset * 0.618) * 0.5 + 0.5; // Golden ratio offset
		
		// Sharp pulses with cubic easing
		pulse1 = pow(pulse1, 3.0);
		pulse2 = pow(pulse2, 3.0);
		
		// Combine pulses
		float combinedPulse = max(pulse1, pulse2);
		
		// Pulse intensity scales with how deep we are in the transition
		float transitionDepth = smoothstep(transitionStart, 1.0, colorBlend + transitionStart);
		float pulseIntensity = combinedPulse * transitionDepth * 1.2; // More intense pulse // Increased from 0.5 to 0.8
		
		// Brighten the color during pulses
		currentColor = currentColor * (1.0 + pulseIntensity);
	}
	
	vec3 col = r * currentColor;

	// Feature #6: Apply 3D volumetric depth to rays (uses pre-computed neighbors)
#if RAY_3D_DEPTH
	col = apply3DRayDepth(col, r, neighborRays);
#endif

	// Feature #7: Apply subsurface scattering (uses pre-computed neighbors)
#if RAY_SUBSURFACE
	col = applySubsurfaceScattering(col, r, neighborRays);
#endif

	// Attenuate brightness towards the top
	float brightness = 1.0 - (fragCoord.y / iResolution.xy.y);
	col *= (0.35 + brightness * 0.65);

	// Startup fade-in disabled for debugging
	// if (iTime < 180.0) {
	// 	float startupProgress = iTime / 180.0;
	// 	float startupFade = startupProgress * startupProgress * startupProgress * startupProgress * startupProgress;
	// 	col *= startupFade;
	// }

	return col;
}

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
	// OPTIMIZATION: Compute cached values once per pixel (not per function call)
	g_cursorFactor = computeCursorFactor();
	g_speedVariance = computeSpeedVariance();

	vec2 uv = fragCoord / iResolution.xy;

	vec4 terminalColor = texture(iChannel0, uv);
	
	// Protect bright pixels
	float distFromBlack = length(terminalColor.rgb);
	if (distFromBlack > 0.5) {
		fragColor = terminalColor;
		return;
	}
	
	// For grey-ish pixels, check if they're uniform (UI) or textured (image)
	// Sample neighbors at larger radius to detect texture
	vec2 pixelSize = 1.0 / iResolution.xy;
	float variance = 0.0;
	vec3 centerCol = terminalColor.rgb;
	
	// Simplified variance: just 2 samples (left and right) instead of 4
	vec2 offset1 = vec2(pixelSize.x * 6.0, 0.0);
	vec2 offset2 = vec2(0.0, pixelSize.y * 6.0);
	variance += length(texture(iChannel0, uv + offset1).rgb - centerCol);
	variance += length(texture(iChannel0, uv + offset2).rgb - centerCol);
	variance /= 2.0;
	
	// If there's color variance (image), protect it
	// Solid grey UI has variance near 0
	if (distFromBlack > 0.08 && variance > 0.002) {
		fragColor = terminalColor;
		return;
	}
	
	// Use existing variance for proximity fade (no extra samples)
	float contentProximity = smoothstep(0.0, 0.01, variance);
	float proximityFade = 1.0 - contentProximity * 0.8;

	// OPTIMIZATION: compute rayMask once, reuse for both glitch effects and underwaterRays
	float rawRayMask = underwaterRayMask(fragCoord);
	float rayMask = rawRayMask * proximityFade;
	
	// Detect white-ish colors to reduce effect on text
	float luminance = dot(terminalColor.rgb, vec3(0.299, 0.587, 0.114));
	float whiteMask = smoothstep(0.15, 0.4, luminance); // More protection - starts at 15% luminance (was 20%)
	
	// Detect red colors to protect (like white)
	float redness = terminalColor.r - max(terminalColor.g, terminalColor.b);
	float isRed = smoothstep(0.3, 0.5, redness);
	
	// Detect bright magenta cursor (#ff00f8) and protect it
	float magentaDistance = length(terminalColor.rgb - vec3(1.0, 0.0, 0.973));
	float isCursor = 1.0 - smoothstep(0.0, 0.1, magentaDistance);
	
	// Combine white, red, and cursor protection
	float protectedMask = max(max(whiteMask, isRed), isCursor);
	float adjustedRayMask = rayMask * (1.0 - protectedMask * 0.98);

	// =========================================================================
	// VAPORWAVE PURPLE DETECTION
	// The vaporwave aesthetic embraces the full purple-magenta-pink spectrum:
	// - Neon purple (#ab60ed) - the synthwave glow
	// - Hot magenta (#ff00f8) - the retro future
	// - Deep purple (#c080d0) - the sunset haze
	// - Muted violet (#aa00e8) - the distant neon
	// Any color where blue+red dominates green lives in this space.
	// =========================================================================

	// Primary detection: colors where red+blue overpower green (the purple/magenta family)
	float purpleness = (terminalColor.r + terminalColor.b) * 0.5 - terminalColor.g;
	float isPurple = smoothstep(PURPLE_DETECT_THRESHOLD, PURPLE_DETECT_THRESHOLD * 3.0, purpleness);

	// Secondary detection: high blue with any red component (catches more violet/indigo)
	float hasViolet = terminalColor.b * 0.7 + terminalColor.r * 0.3 - terminalColor.g * 0.5;
	float isViolet = smoothstep(0.10, 0.23, hasViolet);  // 1.5x more generous

	// Vaporwave anchor colors - directly from VAPORWAVE_PALETTE to stay in sync
	vec3 neonPurple = VAPORWAVE_PALETTE[7];  // #ab60ed - synthwave glow
	vec3 hotMagenta = VAPORWAVE_PALETTE[0];  // #ff00f8 - retro future
	vec3 deepPurple = VAPORWAVE_PALETTE[6];  // #c080d0 - sunset haze
	vec3 mutedViolet = vec3(0.667, 0.000, 0.910);  // #aa00e8 - extra anchor

	// Distance to nearest vaporwave purple (use minimum for widest catch)
	float distNeon = length(terminalColor.rgb - neonPurple);
	float distMagenta = length(terminalColor.rgb - hotMagenta);
	float distDeep = length(terminalColor.rgb - deepPurple);
	float distMuted = length(terminalColor.rgb - mutedViolet);
	float nearestPurpleDist = min(min(distNeon, distMagenta), min(distDeep, distMuted));

	// Generous proximity - anything within 1.2 distance gets some effect (1.5x more generous)
	float purpleProximity = 1.0 - smoothstep(0.0, 1.2, nearestPurpleDist);

	// Combine detection methods: purple formula OR violet formula OR very close to a vaporwave color
	float combinedPurple = max(max(isPurple, isViolet * 0.67), purpleProximity * 0.75);  // 1.5x more generous

	// Final intensity - require actual proximity, no minimum floor
	float purpleIntensity = combinedPurple * purpleProximity;
	float nearPurple = step(0.10, combinedPurple);  // Binary: is or isn't purple (1.5x more generous)
	float nearPurpleIntensity = purpleIntensity * nearPurple;
	
	// Boost glitch effect around purple text (reduced spread)
	float purpleGlitchBoost = 1.0 + nearPurpleIntensity * 1800.0;  // +50% (was 1200)

	// For purple text, bypass whiteMask protection to allow glitch on the text itself
	float purpleBypassMask = mix(adjustedRayMask, rayMask, purpleIntensity * 0.9);

	float effectiveProtectedMask = protectedMask;

	// Reuse variance computed earlier (no second sampling loop)
	float localVariance = variance;

	float boostMask = purpleBypassMask * purpleGlitchBoost;

	// Ensure minimum glitch near purple even without rays
	boostMask = max(boostMask, nearPurpleIntensity * 20.0);
	
	// Skip glitchy effects entirely for protected areas - just use terminal color
	vec3 base;
	if (boostMask < 0.01 && nearPurpleIntensity < 0.01) {
		base = terminalColor.rgb;
	} else {
		base = glitchyColor(uv, fragCoord, boostMask, nearPurpleIntensity, effectiveProtectedMask, localVariance);
	}

	float distToTarget = length(terminalColor.rgb - TARGET_COLOR);
	float backgroundMask = 1.0 - smoothstep(COLOR_TOLERANCE, COLOR_TOLERANCE * 2.0, distToTarget);
	vec3 rays = underwaterRays(fragCoord, rawRayMask);  // Pass pre-computed ray mask
	
#if FORCE_BACKGROUND_BLACK
	// Only force pixels to black if they're very dark (luminance < 10%)
	// Reuses 'luminance' computed earlier to avoid redundant dot product
	float forceBlackMask = backgroundMask * (1.0 - smoothstep(0.0, 0.1, luminance));
	base = mix(base, vec3(0.0), forceBlackMask);
#endif

	// Add rays proportionally to how dark the pixel is
	// Use smoothstep to create hard cutoff - only very dark pixels get rays
	// Reuses 'luminance' computed earlier
	float rayIntensity = 1.0 - smoothstep(0.0, 0.25, luminance); // Full rays below 0% luminance, none above 25%

	// Cursor-based brightness: effects dim when cursor moves, brighten over time
	float cursorBright = cursorBrightness();

	// Apply rays with intensity scaling, modulated by cursor brightness
	vec3 finalRgb = base + rays * rayIntensity * UNDERWATER_RAY_STRENGTH * cursorBright;

	// External ray sweep: light sweeping across from virtual source outside terminal
	float sweepLight = externalRaySweep(uv) * rayIntensity * cursorBright;
	finalRgb += vec3(sweepLight * 0.9, sweepLight * 0.7, sweepLight);  // Warm white sweep

	// Purple activation: subtle purple aura on actual purple pixels only
	// Only apply to pixels that are themselves purple (high purpleIntensity, not just nearby)
	float auraStrength = purpleIntensity * 1.125 * (1.0 - smoothstep(0.05, 0.15, luminance));  // +50% (was 0.75)
	if (auraStrength > 0.1) {  // Higher threshold - only strong purple pixels
		float pulsePhase = iTime * cursorBright * 0.05;  // 20-second cycle, cursor-adjusted
		float pulse = sin(pulsePhase) * 0.5 + 0.5;
		auraStrength *= 0.3 + pulse * 0.2;  // Subtle pulsing
		finalRgb = mix(finalRgb, neonPurple, auraStrength * 0.15);  // Reduced from 0.6
	}

	finalRgb = clamp(finalRgb, 0.0, 1.0);

	// Slowly varying global luminosity reduction using smooth sine
	// Speed scales with cursor stillness
	float lumPhase = iTime * cursorBright * 0.0001;
	float lumFactor = (0.256 + sin(lumPhase) * 0.09 + 0.09) * 0.61; // Range: 0.16 to 0.27 (40% dimmer)
	finalRgb *= lumFactor;

	// Feature #4: Grecas overlay (faint stepped spiral pattern on rays)
#if AZTEC_GRECAS
	float grecasOverlay = grecasPattern(uv, rayIntensity) * cursorBright;
	finalRgb += vec3(grecasOverlay * 0.8, grecasOverlay * 0.6, grecasOverlay); // Slight cyan tint
#endif

	// Grey text outline (darken pixels near grey text for contrast)
#if GREY_OUTLINE_ENABLE
	float outlineDarken = greyTextOutline(uv, pixelSize, luminance);
	finalRgb *= (1.0 - outlineDarken);
#endif

	// Green background suppression (darken green-dominant backgrounds)
#if GREEN_BG_SUPPRESS
	float greenExcess = terminalColor.g - max(terminalColor.r, terminalColor.b);
	float isGreenBg = smoothstep(GREEN_BG_THRESHOLD, GREEN_BG_THRESHOLD * 2.0, greenExcess);
	// Only apply to darker pixels (backgrounds, not bright green text)
	// Reuses 'luminance' computed earlier
	float bgMask = 1.0 - smoothstep(0.0, 0.3, luminance);
	finalRgb *= 1.0 - (isGreenBg * bgMask * GREEN_BG_DARKEN);
#endif

#if DEBUG_PURPLE
	// Debug visualization: show purple detection
	if (purpleIntensity > 0.1) {
		finalRgb = mix(finalRgb, vec3(1.0, 0.0, 1.0), 0.5);  // Bright magenta overlay on detected purple
	}
	if (nearPurpleIntensity > 0.1) {
		finalRgb.g += 0.3;  // Green tint where nearPurple is active
	}
#endif

	fragColor = vec4(finalRgb, terminalColor.a);
}
