#!/usr/bin/env bash
# Machine-wide lock around a test run: `scripts/test-lock.sh cargo nextest run ...`.
# Only one full test run per machine at a time (n4w4): N worktrees each running nextest
# with 8 threads multiplied the load. A second run waits, then proceeds.
#
# The lock is a symlink whose target is "<pid> <utc time> <worktree>": symlink creation is
# atomic and carries the holder's details in one step (no flock on macOS, no half-written
# lock). A lock whose pid is dead is stale and taken over. Exit status is the command's.
# BRIDLE_TEST_NOLOCK=1 skips the lock (CI, emergencies).
set -u

if [ "$#" -eq 0 ]; then
    echo "usage: test-lock.sh <command> [args...]" >&2
    exit 2
fi
if [ "${BRIDLE_TEST_NOLOCK:-}" = 1 ]; then
    exec "$@"
fi

lock="${BRIDLE_TEST_LOCK:-$HOME/.bridle/test.lock}"
mkdir -p "$(dirname "$lock")"
me="$$ $(date -u +%Y-%m-%dT%H:%M:%SZ) $(pwd)"
held=0

release() {
    # Only remove a lock that is still ours.
    if [ "$held" = 1 ] && [ "$(readlink "$lock" 2>/dev/null)" = "$me" ]; then
        rm -f "$lock"
    fi
    held=0
}
trap release EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

told=0
while ! ln -s "$me" "$lock" 2>/dev/null; do
    holder=$(readlink "$lock" 2>/dev/null) || continue # released between the two calls
    pid=${holder%% *}
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "test lock held by dead pid $pid; taking it over" >&2
        # Remove only the lock we inspected; if another waiter already replaced it, retry.
        [ "$(readlink "$lock" 2>/dev/null)" = "$holder" ] && rm -f "$lock"
        continue
    fi
    if [ "$told" = 0 ]; then
        rest=${holder#* }
        echo "waiting for the test lock held by ${rest#* } since ${rest%% *} (pid $pid)" >&2
        told=1
    fi
    sleep "${BRIDLE_TEST_LOCK_POLL:-5}"
done
held=1

"$@"
