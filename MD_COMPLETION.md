## Zsh Markdown TUI Integration (Authoritative Instructions)

When asked to modify shell configuration related to Markdown browsing or viewing:

### Goals
- Preserve normal shell behavior.
- Avoid prompt hooks, aliases that override binaries, or global side effects.
- Prefer Zsh-native mechanisms (ZLE widgets, suffix aliases, shell functions).
- Trigger advanced behavior only on explicit user action.
- Optimize for discoverability without automation surprises.

### Viewing Markdown
- Use `glow -p` as the canonical Markdown renderer.
- Never replace `glow` with an alias unless explicitly instructed.
- Prefer shell functions over aliases when default flags are required.

### Lightweight Access Patterns
Implement, when requested:
- A suffix alias so executing a `.md` filename renders via `glow -p`
- A short helper command (`md`) that renders one or all Markdown files in the current directory
- These must not interfere with `cat`, `vim`, or other tools

### TUI Browsing
- Do **not** use `fzf` for multi-column navigation
- Treat `fzf` as a selector only, never a browser
- For multi-column, navigable layouts, use a dedicated TUI file manager

Preferred tools (in order):
1. `yazi`
2. `lf`
3. `broot` (tree-oriented fallback)

### Columnar Browser Integration
When wiring a browser:
- Use it as an explicit, user-triggered action
- Configure Markdown preview using `glow -p`
- Ensure navigation between panes is possible
- Avoid background daemons or filesystem watchers

### ZLE / TAB Behavior
If implementing keybindings:
- Bind only via ZLE widgets
- TAB behavior must:
  - Fall back to normal completion when the buffer is non-empty
  - Activate only when Markdown files exist
  - Launch a browser, not a selector
- Degrade gracefully and invisibly when conditions are not met

### Non-Goals
- No automatic rendering on `cd`
- No prompt modification
- No global overrides of `ls`, `cat`, or core utilities
- No implicit behavior based solely on directory contents

### Style Constraints
- Prefer small, composable shell functions
- Keep all changes reversible
- Favor clarity over cleverness
- If tradeoffs exist, choose the option least likely to surprise the user later

## Zsh + Markdown Browser (Imperative Execution Instructions)

When instructed to apply these changes, perform the following actions exactly.

---

### 1. Zsh: Markdown Rendering Defaults

Modify `~/.zshrc`:

#### a) Ensure `glow` default flags use a shell function
Add or replace:

```zsh
glow() {
  command glow -p "$@"
}

