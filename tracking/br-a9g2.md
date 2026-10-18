+++
id = "br-a9g2"
title = "agent spawn --allow-tool silently does nothing for a tool outside the role's --tools set (WebSearch on a worker)"
kind = "fix"
state = "integrated"
created_at = "2026-10-07T00:11:13.981Z"
updated_at = "2026-10-18T02:47:44.396274Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
    "agent:wa9g2",
]
branch = "bridle/wa9g2"
commit = "648754040d66a0c3c12e15e6d48e8ee83e7f1c94"
summary = """Chose the refusal (per orchestrator on the ticket): spawn_agent in supervisor.rs refuses a bare built-in --allow-tool name missing from the role's --tools set with "<tool> isn't in the <role> role's tool set; use the researcher role". Patterns like Bash(git *) are not checked; roles with no tools set are unaffected. Tests: new spawn_extra_allowed_tool_outside_role_tools_is_refused; two existing tests that used WebSearch on a worker now use Glob. Documented in docs/design/cli.md and CHANGELOG."""
ticket = "a9g2"
+++

docs/tickets/open/agent-spawn-allow-tool-silently-does-nothing-for-a-tool-outs-a9g2.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.766Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T07:22:56.771Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-17T15:26:07.290Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · external:orchestrator · 2026-10-17T23:54:00.582Z
orchestrator: take the refusal option. Refuse the spawn with '<tool> isn't in the <role> role's tool set; use the researcher role'. Adding the tool to --tools would widen a role's tool set from the spawn line, and the human's standing decision (bridle role file) holds anything that widens an agent's permissions for them. If the refusal turns out too strict, file that as a separate ticket for the human.

### note · agent:wa9g2 · 2026-10-18T00:53:17.980Z
built: 16a14573d94f40924d27bb7f97ac1f60d39498ee (branch bridle/wa9g2)

### note · agent:wa9g2 · 2026-10-18T00:53:18.386Z
done: --allow-tool for a built-in tool outside the role's --tools is refused (naming researcher); just check exit 0, 1835 tests; checked on 16a14573's parent commit, merge of main only touched an unrelated gateway test; 16a14573d94f40924d27bb7f97ac1f60d39498ee

### note · agent:wa9g2 · 2026-10-18T00:53:39.223Z
CHANGELOG fixed to one refusal line; cli.md already says refusal; docs-only change on top of the green check (1835 passed); 20a8bb0ab138cd64d028b87bf3df9dcc174b874a

### note · agent:manager-2 · 2026-10-18T02:47:04.034Z
integrated: 648754040d66a0c3c12e15e6d48e8ee83e7f1c94 (branch bridle/wa9g2)

### note · agent:manager-2 · 2026-10-18T02:47:19.294Z
cleanup: removed agent wa9g2, branch bridle/wa9g2

### note · external:advisor/product-manager · 2026-10-18T02:47:44.396Z
advisor product-manager: resolved. Verified on main (64875404 has the worker's last commit 20a8bb0a): spawn refuses a built-in tool outside the role's --tools, naming the researcher role; test in spawn_messaging_test.rs; cli.md and CHANGELOG say so.
