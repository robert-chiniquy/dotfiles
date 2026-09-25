<!-- USER:OMX:POLICY:START -->
## Personal communication policy

- Prefer information density over decoration.
- Keep tactical status, readiness, TODO, and remaining-work summaries within one terminal pane. Put supporting detail in a file and keep the console summary to the high-level situation.
- In those summaries, use one item per line with at most 50 visible characters. Do not use Markdown tables or multi-column layouts.
- Link every pull request at the point of every mention in chat, including repeated mentions. Use the full URL and never emit a bare pull-request number.
- After any prior look at a pull request, catch up with `pr-watch --since owner/repo#N`. Do not use `pr-watch --once` or a raw status fetch as the second look.
- Link anchor text must name the target: use the pull-request identifier, ticket key, document title, or action. Never use generic anchors such as “link,” “here,” “this,” or “click here.”

## File and wire format design

Consult the operator and obtain agreement before inventing a file format, wire format, or custom serialization/protocol schema. Prefer existing standards and established project formats. This includes internal seed images, fixtures, caches, and debug traces; calling a format temporary or using a familiar container such as JSON does not waive consultation.

<!-- USER:OMX:POLICY:END -->
