#!/usr/bin/env python3
"""fake-claude.py — stdlib-only stand-in for `claude -p --input-format
stream-json --output-format stream-json`, used by bridle-claude's (and later
bridle-daemon's) tests so a supervisor can be exercised without spending
tokens on the real thing. Not a full emulation: just enough of the verified
protocol (docs/spikes/01-stream-json-findings.md) to drive turns, folds,
interrupts and shutdown.

Usage:
    fake-claude.py [--session-id ID | --resume ID] [--replay-user-messages]
                    [any other claude flag, ignored]

Reads stream-json user messages on stdin, one per line. Most text just gets
echoed back as "echo: TEXT" in an assistant turn. A few magic words in the
message text (matched exactly, or as a prefix where noted) change behaviour:

    SLEEP n       Run a fake Bash `sleep n` tool call. While "running", watch
                  stdin for more input: a plain user message is folded into
                  this turn (echoed once the tool finishes, and appended to
                  the reply text); an interrupt control_request aborts the
                  turn immediately (control_response, a synthetic failed
                  tool_result, the "[Request interrupted...]" marker, then an
                  error_during_execution result). Messages that were folded
                  in but not yet consumed when the interrupt lands each run
                  as their own next turn, in order.
    EXIT n        Exit immediately with code n. No result is emitted.
    CRASH         Write a line to stderr and exit(3) immediately. No result.
    SPAWN_CHILD   Start a detached `sleep 300` in a new session (its own
                  session id, for orphan-sweep tests) and put its pid in the
                  reply text ("echo: SPAWN_CHILD pid=1234").
    DENY          Like a plain turn, but the result carries a synthetic
                  permission_denials entry.

`--max-budget-usd X` behaves like real claude (docs/spikes/02-budget-cap-findings.md):
spend is per process, checked after the turn's model work, so the turn that
crosses the cap still runs but its result is `error_max_budget_usd`; every
later turn fails the same way at once, costing nothing.

`system/init` reports `claude_code_version` from a `.fake-claude-version` file
in the working directory, or "fake".

Any other control_request gets a generic success control_response echoing
its subtype. On stdin EOF, the current turn (if any) finishes, then the
process exits 0 if the last result wasn't an error, 1 otherwise — matching
real claude (docs/spikes/01-stream-json-findings.md, S5). SIGTERM exits 143.
"""

import json
import queue
import signal
import subprocess
import sys
import threading
import time
import uuid

USAGE = {
    "input_tokens": 10,
    "output_tokens": 5,
    "cache_creation_input_tokens": 0,
    "cache_read_input_tokens": 100,
}

TIMEOUT = object()

state = {
    "session_id": None,
    "cost": 0.0,
    "last_error": False,
    "msg_counter": 0,
    "rate_limit_emitted": False,
    "process_cost": 0.0,
    "max_budget": None,
}

replay = False
q: "queue.Queue" = queue.Queue()


def emit(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


def rate_limit_event():
    return {
        "type": "rate_limit_event",
        "rate_limit_info": {
            "status": "allowed",
            "resetsAt": 1790533200,
            "rateLimitType": "five_hour",
            "unifiedWindows": {
                "five_hour": {"utilization": 0.05, "resetsAt": 1790533200},
                "seven_day": {"utilization": 0.13, "resetsAt": 1790751600},
            },
        },
    }


def emit_assistant(content):
    """Emits one assistant event, then (once per process, after the very
    first assistant output) the rate_limit_event, matching real claude's
    "once per process, on the first API response" behaviour."""
    state["msg_counter"] += 1
    emit(
        {
            "type": "assistant",
            "message": {"id": f"msg_{state['msg_counter']}", "content": content},
            "parent_tool_use_id": None,
            "session_id": state["session_id"],
        }
    )
    if not state["rate_limit_emitted"]:
        emit(rate_limit_event())
        state["rate_limit_emitted"] = True


def emit_user(content):
    emit(
        {
            "type": "user",
            "message": {"role": "user", "content": content},
            "parent_tool_use_id": None,
            "session_id": state["session_id"],
        }
    )


def bump_cost():
    state["cost"] = round(state["cost"] + 0.001, 3)
    state["process_cost"] = round(state["process_cost"] + 0.001, 3)


def over_budget():
    cap = state["max_budget"]
    return cap is not None and state["process_cost"] >= cap


def emit_budget_result():
    emit(
        {
            "type": "result",
            "subtype": "error_max_budget_usd",
            "is_error": True,
            "num_turns": 1,
            "session_id": state["session_id"],
            "total_cost_usd": state["cost"],
            "usage": USAGE,
            "terminal_reason": "budget_exhausted",
            "errors": [f"Reached maximum budget (${state['max_budget']})"],
            "permission_denials": [],
        }
    )
    state["last_error"] = True


def claude_code_version():
    try:
        with open(".fake-claude-version") as f:
            return f.read().strip() or "fake"
    except OSError:
        return "fake"


def reader():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            q.put(json.loads(line))
        except json.JSONDecodeError:
            continue
    q.put(None)


def wait_item():
    """Blocks until an item (or the `None` EOF marker) is available. Polls
    in short slices so SIGTERM is always noticed promptly regardless of
    where the process is blocked."""
    while True:
        try:
            return q.get(timeout=0.2)
        except queue.Empty:
            continue


def wait_item_until(deadline):
    """Like `wait_item`, but returns the `TIMEOUT` sentinel once `deadline`
    (a `time.monotonic()` value) passes with nothing queued."""
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return TIMEOUT
        try:
            return q.get(timeout=min(0.2, remaining))
        except queue.Empty:
            continue


def handle_generic_control_request(item):
    req_id = item.get("request_id")
    subtype = item.get("request", {}).get("subtype")
    emit(
        {
            "type": "control_response",
            "response": {"subtype": "success", "request_id": req_id, "response": {"subtype": subtype}},
        }
    )


def command_line(text):
    """The magic word bridle actually means, from a possibly multi-line
    message: bridle wraps and prefixes real messages (the `[bridle message
    ...]` envelope, the spawn prompt's "You are ... in ..." fact line), so
    the literal magic word is whatever the last non-empty line is, not
    necessarily the whole stdin text."""
    lines = [line for line in text.splitlines() if line.strip()]
    return lines[-1] if lines else text


def run_turn(text):
    sid = state["session_id"]
    if replay:
        emit_user(text)
    emit(
        {
            "type": "system",
            "subtype": "init",
            "session_id": sid,
            "model": "fake-haiku",
            "tools": ["Bash"],
            "capabilities": ["interrupt_receipt_v1"],
            "claude_code_version": claude_code_version(),
        }
    )

    if over_budget():
        emit_budget_result()
        return

    command = command_line(text)

    if command == "CRASH":
        print("fake-claude: CRASH requested", file=sys.stderr, flush=True)
        sys.exit(3)
    if command.startswith("EXIT "):
        sys.exit(int(command.split(None, 1)[1]))

    folded = []
    aborted = False
    spawned_pid = None

    if command.startswith("SLEEP "):
        seconds = float(command.split(None, 1)[1])
        emit_assistant([{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": f"sleep {seconds}"}}])
        deadline = time.monotonic() + seconds
        while True:
            item = wait_item_until(deadline)
            if item is TIMEOUT:
                break
            if item is None:
                # stdin EOF mid-tool: let the tool finish naturally (the
                # turn still completes), remember to exit afterwards.
                state["eof"] = True
                continue
            if item.get("type") == "control_request" and item.get("request", {}).get("subtype") == "interrupt":
                req_id = item["request_id"]
                emit(
                    {
                        "type": "control_response",
                        "response": {"subtype": "success", "request_id": req_id, "response": {"still_queued": []}},
                    }
                )
                emit_user(
                    [
                        {
                            "type": "tool_result",
                            "tool_use_id": "t1",
                            "content": "The user doesn't want to proceed with this tool use.",
                            "is_error": True,
                        }
                    ]
                )
                emit_user([{"type": "text", "text": "[Request interrupted by user for tool use]"}])
                aborted = True
                break
            if item.get("type") == "user":
                folded.append(item["message"]["content"])
                continue
            if item.get("type") == "control_request":
                handle_generic_control_request(item)
                continue
            # Unrecognized item shape: ignore and keep waiting.

        if aborted:
            bump_cost()
            emit(
                {
                    "type": "result",
                    "subtype": "error_during_execution",
                    "is_error": True,
                    "num_turns": 1,
                    "session_id": sid,
                    "total_cost_usd": state["cost"],
                    "usage": USAGE,
                    "terminal_reason": "aborted_tools",
                    "stop_reason": "tool_use",
                    "permission_denials": [],
                }
            )
            state["last_error"] = True
            # Messages folded in before the interrupt landed, but not yet
            # consumed, each run as their own next turn.
            for folded_text in folded:
                run_turn(folded_text)
            return

        emit_user(
            [{"type": "tool_result", "tool_use_id": "t1", "content": "(Bash completed with no output)", "is_error": False}]
        )
        for folded_text in folded:
            if replay:
                emit_user(folded_text)

    elif command == "SPAWN_CHILD":
        proc = subprocess.Popen(
            ["sleep", "300"],
            start_new_session=True,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        spawned_pid = proc.pid

    reply = f"echo: {text}"
    if spawned_pid is not None:
        reply = f"echo: {text} pid={spawned_pid}"
    if folded:
        reply = reply + "\n" + "\n".join(f"echo: {t}" for t in folded)

    emit_assistant([{"type": "text", "text": reply}])

    denials = []
    if command == "DENY":
        denials = [{"tool_name": "Bash", "tool_use_id": "t1", "tool_input": {"command": "rm -rf /"}}]

    bump_cost()
    if over_budget():
        emit_budget_result()
        return
    emit(
        {
            "type": "result",
            "subtype": "success",
            "is_error": False,
            "num_turns": 1,
            "result": reply,
            "session_id": sid,
            "total_cost_usd": state["cost"],
            "usage": USAGE,
            "terminal_reason": "completed",
            "stop_reason": "end_turn",
            "permission_denials": denials,
        }
    )
    state["last_error"] = False


def parse_args(argv):
    session_id = None
    resume = False
    replay_flag = False
    variadic = {"--allowedTools", "--allowed-tools", "--disallowedTools", "--disallowed-tools", "--add-dir"}
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--session-id" and i + 1 < len(argv):
            session_id = argv[i + 1]
            i += 2
            continue
        if a == "--resume" and i + 1 < len(argv):
            session_id = argv[i + 1]
            resume = True
            i += 2
            continue
        if a == "--max-budget-usd" and i + 1 < len(argv):
            state["max_budget"] = float(argv[i + 1])
            i += 2
            continue
        if a == "--replay-user-messages":
            replay_flag = True
            i += 1
            continue
        if a in variadic:
            # Variadic: consume tokens until the next flag.
            i += 1
            while i < len(argv) and not argv[i].startswith("-"):
                i += 1
            continue
        if a.startswith("-"):
            # Unknown flag: best-effort consume its value if the next token
            # doesn't itself look like a flag.
            if i + 1 < len(argv) and not argv[i + 1].startswith("-"):
                i += 2
            else:
                i += 1
            continue
        i += 1
    if session_id is None:
        session_id = str(uuid.uuid4())
    return session_id, resume, replay_flag


def main():
    global replay
    session_id, _resume, replay_flag = parse_args(sys.argv[1:])
    state["session_id"] = session_id
    state["eof"] = False
    replay = replay_flag

    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))

    threading.Thread(target=reader, daemon=True).start()

    while not state["eof"]:
        item = wait_item()
        if item is None:
            state["eof"] = True
            break
        if item.get("type") == "user":
            run_turn(item["message"]["content"])
        elif item.get("type") == "control_request":
            handle_generic_control_request(item)
        # else: ignore anything else.

    sys.exit(1 if state["last_error"] else 0)


if __name__ == "__main__":
    main()
