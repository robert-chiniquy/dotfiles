---
name: refine-illustrations-iteratively
description: Refine AI-generated illustrations through successive, controlled visual adjustments while preserving approved details and character continuity. Use for iterative image-generation or image-editing sessions involving follow-up directions such as changing composition, viewpoint, aspect ratio, pose, anatomy, wardrobe, props, facial visibility, typography, atmosphere, or removing unwanted artifacts from the latest image.
---

# Refine Illustrations Iteratively

Build the final image as a sequence of controlled deltas. Treat the latest accepted image as the source of truth and preserve everything the user did not ask to change.

## Maintain a visual state

Track three compact sets after every turn:

- **Locked:** approved elements that must survive future edits.
- **Delta:** the specific variables requested in the current turn.
- **Watch:** details likely to regress when applying the delta.

Infer locks from approval language such as "perfect," "great," "this is good," or a request that changes only one detail. Do not ask the user to restate established attributes.

## Apply the refinement loop

1. **Anchor the scene.** Establish subject, setting, mood, medium, and central visual idea.
2. **Solve structure.** Adjust viewpoint, framing, aspect ratio, camera distance, placement, motion, geometry, and major pose before fine styling.
3. **Stabilize identity.** Preserve the number of figures and their distinguishing traits, silhouette, proportions, hair, headwear, face visibility, and relationships to one another.
4. **Tune styling and props.** Refine clothing, fabric, accessories, vehicles, devices, colors, and material details without redesigning locked elements.
5. **Add atmosphere and typography late.** Introduce lighting, texture, glow, ornament, and exact text after the composition is stable. Preserve exact spelling, capitalization, line breaks, placement, and treatment.
6. **Run a cleanup pass.** Remove unintended labels, patches, extra limbs, duplicate props, stray text, or other artifacts while leaving the rest untouched.

Move forward or backward in this order when the user requests it; do not force the user through explicit stages.

## Translate each request into a minimal edit

Write the image-edit instruction as:

> Preserve [locked state]. Change only [delta]. Ensure [watch items].

Use positive spatial and visual language. For example, translate "not sideways" into "facing forward into the turn," while retaining the exclusion when it prevents a common regression.

When several changes arrive together, group them by dependency:

- composition and camera;
- pose and anatomy;
- identity and continuity;
- wardrobe and objects;
- lighting, text, and cleanup.

Resolve structural dependencies first inside the same edit instruction.

## Preserve continuity aggressively

- Use the most recent image as the edit reference whenever available.
- Reassert fragile locked details that image models commonly lose during nearby edits.
- Keep recurring characters recognizable through silhouette and a short identity ledger rather than bloating every prompt.
- Treat aspect ratio requests as framing changes, not permission to redesign the scene.
- Treat "remove X" as a surgical cleanup: remove X and preserve the surrounding garment, texture, lighting, and composition.
- Do not silently revive details the user previously removed or rejected.

## Handle ambiguous feedback

Interpret a narrow follow-up as a narrow edit. If the request conflicts with an established lock, follow the newest explicit instruction and update the lock. Ask only when two materially different visual interpretations remain and choosing one would reshape the result.

## Keep interaction lightweight

Generate or edit directly when the request is actionable. After each result, accept natural-language corrections without demanding a full prompt. Carry the visual state forward internally and let the user refine by saying things like "make me skinnier," "spin it around," "hide our faces," "move the sidecar back," or "remove the patch."
