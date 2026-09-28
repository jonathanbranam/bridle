#!/bin/zsh
# Background watcher for the orchestrator (workflow/base/roles/orchestrator.md). Exits, waking the
# orchestrator, on: a question to the human, a message to the orchestrator, main moving, an unexpected exit/crash/stall, all
# agents idle for 15 min, five_hour >= 93% or seven_day >= 85%, a budget hold starting, a failed CI run on main
# (interim, until bridle reports CI itself: ticket c8qw). Usage: orchestrator-watch.sh <since-seq>
export BRIDLE_TOKEN=$(cat ~/.bridle-orchestrator.token)
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
since=${1:-0}
head0=$(git rev-parse main)
idle_ticks=0
seen_file=~/.bridle-orchestrator-seen-questions
hold_file=~/.bridle-orchestrator-hold-state
ci_seen_file=~/.bridle-orchestrator-ci-seen
ci_ticks=0
while true; do
  U=$(bridle status --json 2>/dev/null | jq -r .daemon.url)
  if [[ -n $U && $U != null ]]; then
    ev=$(bridle events --json --since $since 2>/dev/null)
    last=$(print -r -- "$ev" | jq '[.[].seq] | max // empty')
    [[ -n $last ]] && since=$last
    hit=$(print -r -- "$ev" | jq -c '[.[] | select((.kind=="agent.exited" and .data.reason!="stdin_closed" and .data.reason!="budget_paused") or (.kind=="agent.state" and (.data.to=="crashed" or .data.to=="stalled")))]')
    # Questions stay unread until the human reads them, and the orchestrator can't mark them, so
    # remember the ones already reported.
    q=$(curl -s -H "Authorization: Bearer $BRIDLE_TOKEN" "$U/v1/messages?to=human&unread=true&limit=50" \
      | jq -c --rawfile seen <(cat $seen_file 2>/dev/null) '[.[] | select(.kind=="question" and (.id as $i | ($seen | split("\n") | index($i)) == null))]')
    [[ -n $q && $q != "[]" ]] && print -r -- "$q" | jq -r '.[].id' >> $seen_file
    util=$(bridle status --json | jq '[.rate_limits[]? | if .window=="five_hour" then (.utilization//0)/'"${FIVE_HOUR_WAKE:-0.93}"' else (.utilization//0)/0.85 end] | max')
    if [[ -n $hit && $hit != "[]" ]]; then echo "EVENTS since=$since"; print -r -- "$hit"; exit 0; fi
    if [[ -n $q && $q != "[]" ]]; then echo "QUESTION since=$since"; print -r -- "$q"; exit 0; fi
    # Anything sent to the orchestrator itself: the manager's plans, STOPs and requests for help.
    # Read it with `bridle inbox --mark-read`, or this fires again.
    # Since P0-3b, --json is {messages, open_questions}.
    mine=$(bridle inbox --json 2>/dev/null)
    if [[ -n $mine ]] && print -r -- "$mine" | jq -e '(.messages // .) + (.open_questions // []) | length > 0' >/dev/null 2>&1; then echo "INBOX since=$since"; print -r -- "$mine"; exit 0; fi
    busy=$(bridle agents --json | jq '[.[] | select(.state=="working" or .state=="starting")] | length')
    if (( busy == 0 )); then (( idle_ticks++ )); else idle_ticks=0; fi
    if (( idle_ticks >= 30 )); then echo "ALL IDLE since=$since"; bridle agents; exit 0; fi
    if (( util >= 1.0 )); then echo "USAGE since=$since"; bridle status; exit 0; fi
    # A hold is the maintenance window (the role's "Budget holds"), so wake once when the
    # governor leaves `normal`; the file remembers the state already reported.
    gov=$(bridle budget --json 2>/dev/null | jq -r '.state // empty')
    if [[ -n $gov ]]; then
      if [[ $gov != normal && $gov != $(cat $hold_file 2>/dev/null) ]]; then
        print -r -- $gov > $hold_file; echo "HOLD since=$since"; bridle budget; exit 0
      fi
      [[ $gov == normal ]] && rm -f $hold_file
    fi
  fi
  # Every ~2 minutes: a failed GitHub Actions run on main not reported before.
  if (( ++ci_ticks % 4 == 1 )); then
    failed=$(gh run list --branch main --limit 10 --json databaseId,conclusion,headSha,displayTitle,url 2>/dev/null \
      | jq -c --rawfile seen <(cat $ci_seen_file 2>/dev/null) '[.[] | select(.conclusion=="failure" and ((.databaseId|tostring) as $i | ($seen | split("\n") | index($i)) == null))]')
    if [[ -n $failed && $failed != "[]" ]]; then
      print -r -- "$failed" | jq -r '.[].databaseId' >> $ci_seen_file
      echo "CI FAILED since=$since"; print -r -- "$failed"; exit 0
    fi
  fi
  if [[ $(git rev-parse main) != $head0 ]]; then echo "MAIN MOVED since=$since"; git log --oneline $head0..main; exit 0; fi
  sleep 30
done
