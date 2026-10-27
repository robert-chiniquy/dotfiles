#!/usr/bin/env zsh
# Tests for agent-sessions.zsh: drives an interactive zsh on a pseudo-terminal
# (zsh/zpty) with stub agent-sessions and fzf on PATH and the real jq.
# Run: zsh .config/zsh/tests/agent-sessions.test.zsh

zmodload zsh/zpty
here=${0:A:h}
# No `..` in the stub directory: zsh's HASH_DIRS skips such PATH entries when
# it fills the command hash, and the real fzf would then win over the stub.
work=${here:h:h:h}/scratch/agent-sessions-test
rm -rf $work; mkdir -p $work/bin $work/a $work/b
fail=0
check() { if [[ $2 == 1 ]]; then print "ok   $1"; else print "FAIL $1"; fail=1; fi }

# Stub agent-sessions: `list` logs a call and prints rows chosen by
# $work/mode; `open` logs the ref it was given.
#   cd    one finished row whose resume line cds again (re-enters the hook)
#   int   interrupts the shell mid-list
#   live  one live row (selecting it must call open, not eval)
#   none  no rows
cat > $work/bin/agent-sessions <<EOF
#!/bin/sh
case "\$1" in
  open) echo "open \$2" >> $work/opened; exit 0 ;;
esac
echo x >> $work/calls
case "\$(cat $work/mode)" in
  cd) printf '%s\n' '{"sessions":[{"harness":"claude","id":"s1","live":false,"status":"done","name":"n","cwd":"$work/b","resume":"builtin cd $work/b"}],"sources":[]}' ;;
  live) printf '%s\n' '{"sessions":[{"harness":"grok","id":"g1","live":true,"status":"idle","name":"n","cwd":"$work/a","resume":"echo RESUMED >> $work/opened"}],"sources":[]}' ;;
  int) kill -INT "\$SHELL_PID" ;;
  *) printf '%s\n' '{"sessions":[],"sources":[]}' ;;
esac
exit 0
EOF
# Stub fzf: picks the first line.
printf '#!/bin/sh\nhead -n 1\n' > $work/bin/fzf
chmod +x $work/bin/agent-sessions $work/bin/fzf

# Run each command line in a fresh interactive zsh; return the number of
# list calls. A marker line confirms every command line was processed.
run() {
  rm -rf $work/calls $work/opened $work/cache
  zpty -b t "env PATH=$work/bin:$PATH XDG_CACHE_HOME=$work/cache zsh -f -i"
  zpty -w t "SHELL_PID=\$\$; source $here/../agent-sessions.zsh; PS1=''"
  local line
  for line in "$@"; do zpty -w t "$line"; done
  zpty -w t 'print -r -- DONE$((1+1))'
  # A read that times out leaves $line unset; give up after about 10 s so a
  # hung shell fails its check instead of the loop running forever.
  local out='' tries=0
  while (( tries++ < 200 )); do
    line=''
    zpty -r -t t line 2>/dev/null || sleep 0.05
    out+=$line
    [[ $out == *DONE2* ]] && break
  done
  [[ $out == *DONE2* ]] || print -r -- "${out[-600,-1]}" >&2
  zpty -d t
  [[ -f $work/calls ]] && wc -l < $work/calls | tr -d ' ' || echo 0
}

echo cd >| $work/mode
check NestedCdDoesNotReenterHook $(( $(run "builtin cd $work/a") == 1 ))
echo int >| $work/mode
check InterruptDoesNotDisableLaterPicks $(( $(run "_agent_sessions_cd" "echo none >| $work/mode" "_agent_sessions_cd") == 2 ))
echo none >| $work/mode
# Fails if the hook lists again for a directory it listed under 2 minutes ago.
check SameDirWithinTwoMinutesListsOnce $(( $(run "_agent_sessions_cd" "_agent_sessions_cd" "_agent_sessions_cd") == 1 ))
# Fails if one directory's list is reused for another.
check EachDirGetsItsOwnList $(( $(run "builtin cd $work/a" "builtin cd $work/b" "builtin cd $work/a") == 2 ))
# Fails if a list older than 2 minutes is reused.
check StaleListIsReadAgain $(( $(run "_agent_sessions_cd" \
  'zmodload zsh/datetime; strftime -s ts %Y%m%d%H%M.%S $((EPOCHSECONDS - 180)); touch -t $ts '"$work"'/cache/agent-sessions/cwd/*.json' \
  "_agent_sessions_cd") == 2 ))
# Fails if a cache file jq cannot parse is used instead of listing.
check UnreadableCacheIsReadAgain $(( $(run "_agent_sessions_cd" \
  "print garbage >| $work/cache/agent-sessions/cwd/*.json(N[1])" \
  "_agent_sessions_cd") == 2 ))
check SkipPrefixSkipsList $(( $(run "AGENT_SESSIONS_SKIP=1 builtin cd $work/a") == 0 ))
echo live >| $work/mode
run "_agent_sessions_cd" >/dev/null
[[ "$(< $work/opened)" == "open grok/g1" ]]; check LiveRowCallsOpenNotResume $(( ! $? ))

rm -rf $work
exit $fail
