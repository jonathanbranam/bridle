#!/bin/zsh
# Has the orchestrator's own session crossed another CONTEXT_WAKE (default 140000) tokens since
# last reported? Prints `CONTEXT <tokens>` and exits 0 on a crossing, else exits 1. Ticket c9zm.
# Reads $BRIDLE_HOME/context/<session id> (written by `bridle statusline`), the id from
# ~/.bridle-orchestrator-session (written by the launcher's SessionStart hook, `bridle orchestrator note-session`, until the daemon reads it). The last reported level
# lives in ~/.bridle-orchestrator-ctx-level as "<session> <level>", level = tokens / CONTEXT_WAKE,
# so a wake happens once per crossing, and again at 280K; a drop (/compact) resets it.
home=${BRIDLE_HOME:-$HOME/.bridle}
sess=$(cat ~/.bridle-orchestrator-session 2>/dev/null) || exit 1
[[ -n $sess ]] || exit 1
tokens=$(cat "$home/context/$sess" 2>/dev/null) || exit 1
[[ $tokens == <-> ]] || exit 1
level=$(( tokens / ${CONTEXT_WAKE:-140000} ))
level_file=~/.bridle-orchestrator-ctx-level
[[ -r $level_file ]] && read -r last_sess last_level < $level_file
[[ $last_sess == $sess ]] || last_level=0
if (( level > last_level )); then
  print -r -- "$sess $level" > $level_file
  echo "CONTEXT $tokens"
  exit 0
fi
(( level < last_level )) && print -r -- "$sess $level" > $level_file
exit 1
