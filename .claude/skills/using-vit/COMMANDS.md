# vit Command Reference

## Agent-Only Commands

### `vit init`
Usage: `vit init`

Options:
- `--beacon <url>` - Git URL or vit beacon URI to use instead of auto-detection
- `-v, --verbose` - Show step-by-step details

Output format:
- Text output.
- On success prints `beacon: vit:...`.
- Without `--beacon`, prints status and hints when beacon is missing.

Error conditions:
- Not running in an agent context (`requireAgent()` gate).
- No git remote found when using `--beacon .`.
- Invalid beacon/git URL input.

Examples:
- `vit init`
- `vit init --beacon .`
- `vit init --beacon https://github.com/org/repo.git`

### `vit skim`
Usage: `vit skim`

Options:
- `--did <did>` - Override DID
- `--handle <handle>` - Filter to single handle
- `--limit <n>` - Max results (default: 25)
- `--json` - Output as JSON array
- `-v, --verbose` - Show step-by-step details

Output format:
- `--json`: JSON array of ATProto records.
- Default text mode: per-cap blocks with `ref`, optional `title`, optional `description`, plus trailing hint.

Error conditions:
- Not running in an agent context (`requireAgent()` gate).
- No DID configured.
- No beacon set.
- Session restore/auth failure.

Examples:
- `vit skim --json`
- `vit skim --handle alice.bsky.social --limit 10`

## Agent-Usable Commands

### `vit doctor`
Usage: `vit doctor`

Options:
- None.

Output format:
- Text diagnostics for setup and beacon status.

Error conditions:
- Config read/parse failures.

### `vit config [action] [key] [value]`
Usage: `vit config [action] [key] [value]`

Options:
- None.

Arguments:
- `[action]` - `list` (default), `set`, `delete`
- `[key]`
- `[value]`

Output format:
- `list`: `key=value` lines.
- `set`/`delete`: silent on success.

Error conditions:
- Unknown action.
- Missing required key/value for `set`.
- Missing required key for `delete`.

Examples:
- `vit config`
- `vit config set did did:plc:example`
- `vit config delete did`

### `vit follow <handle>`
Usage: `vit follow <handle>`

Options:
- `-v, --verbose` - Show step-by-step details
- `--did <did>` - Override DID

Output format:
- Text: `following <handle> (<did>)`.

Error conditions:
- No DID configured.
- Handle already followed.
- Handle resolution/session failure.

Example:
- `vit follow alice.bsky.social`

### `vit unfollow <handle>`
Usage: `vit unfollow <handle>`

Options:
- `-v, --verbose` - Show step-by-step details

Output format:
- Text: `unfollowed <handle>`.

Error conditions:
- Handle not in following list.

Example:
- `vit unfollow alice.bsky.social`

### `vit following`
Usage: `vit following`

Options:
- `-v, --verbose` - Show step-by-step details

Output format:
- Text lines: `handle (did)`.
- If empty: `no followings`.

Error conditions:
- Following file read/parse failures.

Example:
- `vit following`

### `vit ship`
Usage: `vit ship --title <title> --description <description> --ref <ref> [--recap <ref>]`

Input:
- Body is required on stdin (pipe or heredoc).

Options:
- `-v, --verbose` - Show step-by-step details
- `--did <did>` - Override DID
- `--title <title>` (required)
- `--description <description>` (required)
- `--ref <ref>` (required) - must match `^[a-z]+-[a-z]+-[a-z]+$`
- `--recap <ref>` (optional) - ref to derive from; must match `^[a-z]+-[a-z]+-[a-z]+$`

Gate:
- Agent-only (`requireAgent()`).

Output format:
- JSON object on success with request/response metadata.

Error conditions:
- Not running in an agent context (`requireAgent()` gate).
- No DID configured.
- Missing required options.
- Missing stdin body.
- Invalid `--ref` or `--recap` format.
- `--recap` target not found.
- Session restore/auth failure.

Examples:
- `vit ship --title "Retry helper" --description "Adds bounded retry utility" --ref retry-helper-utility <<'EOF'`
- `Add bounded retry utility for transient request failures.`
- `EOF`
- `vit ship --title "Patch" --description "Details" --ref fast-cache-invalidation --recap original-cache-ref --did did:plc:example <<'EOF'`
- `Patch details...`
- `EOF`

### `vit beacon <target>`
Usage: `vit beacon <target>`

Arguments:
- `<target>` - Git URL or vit beacon URI

Options:
- `-v, --verbose` - Show step-by-step details

Output format:
- Text: `beacon: lit <uri>` or `beacon: unlit`.

Error conditions:
- Invalid target format.
- Clone/probe failure.

Examples:
- `vit beacon https://github.com/org/repo.git`
- `vit beacon vit:github.com/org/repo`

## Human-Only Commands

### `vit setup`
Usage: `vit setup`

Options:
- None.

Gate:
- `requireNotAgent()`

Output format:
- Text status for tool prerequisites and login state.

Error conditions:
- Running in an agent context.
- Missing required tools (`git` or `bun`).

### `vit login <handle>`
Usage: `vit login <handle>`

Options:
- `-v, --verbose` - Show discovery details
- `--force` - Force re-login, skip session validation

Output format:
- Text prompts and login status (`DID`, `Logged in`).

Error conditions:
- OAuth timeout/failure.
- Callback or token exchange errors.

Notes:
- No `requireNotAgent()` gate in code.
- Requires browser OAuth interaction.

Example:
- `vit login alice.bsky.social --force`

### `vit adopt <beacon> [name]`
Usage: `vit adopt <beacon> [name]`

Arguments:
- `<beacon>`
- `[name]` (optional directory name)

Options:
- `-v, --verbose` - Show step-by-step details

Gate:
- `requireNotAgent()`

Output format:
- Text summary including resolved beacon, destination directory, and next step.

Error conditions:
- Running in an agent context.
- Target directory already exists.
- Invalid beacon input.
- Fork/clone command failure.

Examples:
- `vit adopt vit:github.com/org/repo`
- `vit adopt https://github.com/org/repo my-copy`

### `vit vet <ref>`
Usage: `vit vet <ref>`

Options:
- `--did <did>` - Override DID
- `--trust` - Mark cap as trusted
- `-v, --verbose` - Show step-by-step details

Gate:
- `requireNotAgent()`

Output format:
- Text review output for a matching cap.
- With `--trust`, writes trust record and prints `trusted: <ref>`.

Error conditions:
- Running in an agent context.
- Invalid ref format.
- No DID configured.
- No beacon set.
- No matching cap for beacon/ref.
- Session restore/auth failure.

Examples:
- `vit vet fast-cache-invalidation`
- `vit vet fast-cache-invalidation --trust`
