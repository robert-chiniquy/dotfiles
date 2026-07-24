# old/ — retired code

Superseded code, kept as a fossil record rather than deleted.

## iterm-session-snapshot (zsh)

**What it was.** A zsh script that inventoried open iTerm2 tabs (cwd + harness
+ agent session id) and emitted a script to reopen a window with a tab per
session, resuming the claude/codex session in each. Lived at
`bin/iterm-session-snapshot`.

**Why it was retired.** Two structural problems, neither fixable within a
shell script cleanly:

1. **Harness detection required the agent to be live.** It read the foreground
   process on each tty to decide claude vs codex. A session that had exited (or
   any tab snapshotted after a reboot/migration — exactly when you need to
   restore) showed up as a bare shell with no resume command. It only worked
   when everything was already running.
2. **No disambiguation for duplicate cwds, and fragile parsing.** Two tabs in
   the same directory both resolved to that cwd's single "newest transcript".
   Session-id extraction leaned on brittle `sed`/`grep`/`xargs` pipelines that
   broke on edge cases (uuid hyphens, trailing-quote cwd matching).

**Antipattern.** Deriving durable state (which session belongs to which tab)
from a transient signal (the live foreground process), and hand-rolling
structured-data parsing in shell pipelines instead of typed, tested code.

**What replaced it.** `iterm-restore` (Rust crate at `iterm-restore/`).
Resolves each tab from the transcripts on disk, ranked by real in-content
last-activity timestamp (file mtimes are unreliable after a migration copy);
assigns distinct sessions across duplicate-cwd tabs and flags them; resumes
in place by tty; `--list` / `--new-window` modes; parsing logic is unit-tested.
It also attempts an exact live-process mapping via `lsof`, but that is
best-effort only — claude closes its transcript between writes, so there is no
durable on-process session signal to rely on.
