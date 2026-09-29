#!/bin/zsh
# Checks scripts/context-check.sh's crossing logic with fake input. Run: scripts/test-context-check.sh
export HOME=$(mktemp -d) BRIDLE_HOME=$(mktemp -d) CONTEXT_WAKE=1000
mkdir $BRIDLE_HOME/context
print s1 > ~/.bridle-orchestrator-session
chk=$(dirname $0)/context-check.sh
t() { print -r -- $2 > $BRIDLE_HOME/context/s1; out=$($chk); rc=$?; [[ $rc == $3 && $out == $4 ]] || { echo "FAIL $1: rc=$rc out=$out"; exit 1; }; }
t below 900 1 ""
t cross 1100 0 "CONTEXT 1100"
t "same level, no repeat" 1500 1 ""
t "next threshold" 2100 0 "CONTEXT 2100"
t "compacted" 300 1 ""
t "crosses again" 1200 0 "CONTEXT 1200"
print s2 > ~/.bridle-orchestrator-session; print 1200 > $BRIDLE_HOME/context/s2
[[ $($chk) == "CONTEXT 1200" ]] || { echo "FAIL new session"; exit 1; }
echo ok
