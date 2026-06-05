# ~/bin/

Custom binaries and tools installed outside of package managers.

## Installed Tools

| Tool | Source | Purpose |
|------|--------|---------|
| codex-capped | Shell script | Run Codex with the default lower heap cap |
| codex-limited | Shell script | Launch Codex with a lower Node heap cap via `NODE_OPTIONS` |
| conns | Shell script | Show active network connections (listening + established) |
| fclones | `cargo install fclones --root ~/` | Fast duplicate file finder |
| gist | Shell script | Create GitHub gist from file(s) or stdin |
| stashshow | Shell script | Show contents of a git stash with diff |
| jwt | Shell script | Decode JWT tokens, show header/payload/expiry |
| uncommit | Shell script | Undo last commit(s) keeping changes staged |
| unpushed | Shell script | Scan repos for uncommitted/unpushed work |
| wt | Shell script | Git worktree helper (list/add/remove) |
| vaporwave-overlay | Custom build | Desktop visual overlay |
| vaporwave-restart | Shell script | Restart overlay |
| vaporwave-stop | Shell script | Stop overlay |
| vw | Shell script | Toggle vaporwave overlay on/off (Swift/Metal) |
| vwr | Shell script | Toggle vaporwave overlay on/off (Rust/wgpu) |
| vw-compare | Shell script | Compare CPU usage between Swift and Rust versions |

## Installation

Cargo tools can be reinstalled via:
```
~/repo/dotfiles/cargo-tools.sh
```

## PATH

Added to PATH in `~/.zprofile`:
```
export PATH="$HOME/bin:$PATH"
```

## Codex Memory Limit

`codex-limited` defaults to `3072 MiB` for Node old-space and forwards all arguments to the real Codex binary.
`codex-capped` runs that default capped launch directly.

Examples:

```bash
codex-capped
codex-limited
CODEX_MAX_OLD_SPACE_MB=4096 codex-limited
CODEX_REAL_BIN=/opt/homebrew/bin/node codex-limited -p 'require("node:v8").getHeapStatistics().heap_size_limit'
```
