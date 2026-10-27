# On cd, offer the agent sessions of the git repository containing $PWD.
#
# agent-sessions lists Claude Code, Codex and Grok sessions in the repository
# (local sources only, no PR or issue lookups, so the cd stays fast). fzf
# reads the rows on stdin and keys from /dev/tty. A live session is focused in its iTerm2 tab with
# `agent-sessions open`; a finished one resumes in THIS shell by evaluating
# its `resume` line (`cd '<cwd>' && <harness> --resume <id>`). No sessions,
# or no git repository, is a silent no-op.
#
# The list is cached per directory under ~/.cache/agent-sessions/cwd/ for 2
# minutes, so moving between directories does not re-read every store.
#
# AGENT_SESSIONS_SKIP=1 on a command returns before fzf. A prefix assignment
# ends with that command. An exported value skips every later cd.

_agent_sessions_cd() {
  [[ -n ${AGENT_SESSIONS_SKIP-} ]] && return  # iterm-restore sets this on its cd
  [[ -o interactive ]] || return             # interactive shells only
  [[ -n $_AGENT_SESSIONS_BUSY ]] && return    # no re-entry (a pick does its own cd)
  [[ -t 1 ]] || return                         # a real terminal on stdout
  command -v agent-sessions >/dev/null 2>&1 || return
  command -v fzf >/dev/null 2>&1 || return
  command -v jq >/dev/null 2>&1 || return

  # local, not global: zsh scoping still hides the flag from the nested cd's
  # hook, and an interrupt or a suspended resumed agent cannot leave it set.
  local _AGENT_SESSIONS_BUSY=1
  local json rows sel ref
  # The list for this directory is reused for 2 minutes: the file holds the
  # `list --json` output unchanged, and (N.mm-2) matches it only while it was
  # written less than 2 minutes ago. A file jq cannot read counts as absent.
  local dir=${XDG_CACHE_HOME:-$HOME/.cache}/agent-sessions/cwd
  local file=$dir/${PWD//\//%}.json
  local fresh=( $file(N.mm-2) )
  if (( $#fresh )) && json="$(< $file)" && print -r -- $json | jq -e . >/dev/null 2>&1; then
    :
  else
    json="$(command agent-sessions list --repo "$PWD" --source claude,codex,grok \
      --no-links --json 2>/dev/null)" || return
    mkdir -p $dir && print -r -- $json >| $file.$$ && mv -f $file.$$ $file
  fi
  # One line per session: action, ref, status, label, cwd (tab separated).
  rows="$(print -r -- $json | jq -r '.sessions[]
    | [ (if .live then "open" else "resume" end),
        .harness + "/" + .id,
        .status,
        ((.name // .recap // .last_prompt // "") | gsub("[\t\n]"; " ") | .[0:60]),
        .cwd ] | @tsv')"
  [[ -n $rows ]] || return

  sel="$(print -r -- $rows | fzf --height 40% --reverse --delimiter '\t' \
    --with-nth 1,3,4,5 --prompt 'agent session> ')" || return
  ref=${${sel#*$'\t'}%%$'\t'*}
  if [[ $sel == open$'\t'* ]]; then
    command agent-sessions open "$ref"
  else
    eval "$(print -r -- $json | jq -r --arg ref $ref \
      '.sessions[] | select(.harness + "/" + .id == $ref) | .resume')"
  fi
}

autoload -U add-zsh-hook
add-zsh-hook chpwd _agent_sessions_cd
