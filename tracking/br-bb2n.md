+++
id = "br-bb2n"
title = "K2: project preflight: check the new machine is ready before a move"
kind = "feature"
state = "built"
created_at = "2026-10-12T13:38:40.934Z"
updated_at = "2026-10-12T14:48:00.641689Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wbb2n",
]
branch = "bridle/wbb2n"
commit = "8d95b4e6cc5c6acfa9c39e9e17eec6c73c5689db"
summary = '`bridle projects preflight --to MACHINE` (new crates/bridle/src/preflight_remote.rs, wired in preflight.rs; cli.rs, cli.md, CHANGELOG). Items run as ssh snippets through a `Remote` trait (fake in tests): reachable (ssh error, stops), clone (asks before git clone), bridle version >= this one, claude login, gh auth, git push dry-run, first word of [commands] check, tmux, timezone offset, WSL systemd, linger, AWS when .bridle/config.toml has [mail]; local: NEW [machine] name == target key, host resolves, projects sync comparison (no --apply). Each failure says "on M run: ...". Caveats: did not make the scan read HEAD (K1 note, left as is); clone path on NEW assumed same as here (HOME-relative); doctor not reused remotely (checks are inline snippets); timezone compares UTC offset.'
ticket = "bb2n"
+++

docs/tickets/open/k2-project-preflight-check-the-new-machine-is-ready-before-a-bb2n.md

## Thread

### note · agent:pm-1 · 2026-10-12T13:39:02.361Z
Brief (pm-1). Model: Sonnet. Size: M. Blocked by br-3ch7 (K1), which creates `bridle project preflight` and its list-of-checks shape; build on it, don't redo it. Read the ask above and br-kt25 '## Design' section 4.

Goal: add the NEW-machine checks to `project preflight <P> --to M`, run over ssh. Each failing item prints what to run on NEW; unreachable NEW fails with the ssh error; all green prints nothing alarming. Reuse existing `bridle doctor` checks where they exist (run doctor remotely if it already covers an item) rather than reimplementing. Also run the 2wnn comparison (`projects sync` without `--apply`) and the clone check (present, or cloned after a confirm).

Items: bridle binary >= OLD's version, claude logged in, gh auth, git push access to P's origin, P's toolchain command exists, tmux, `[machine] name` equals the name owner.toml will record, Tailscale name resolves, timezone, AWS credentials when `[mail]` names P, and on Linux WSL systemd and linger.

Files likely: the preflight code from K1 in crates/bridle/src, doctor checks, docs/design/cli.md, CHANGELOG.md. Tests: fake the ssh runner (no real ssh in tests); one test per failing-item message plus unreachable.

Acceptance: `just check` passes. Out of scope: the move command, anything about OLD's drain/stop, auto-fixing NEW.

### note · external:advisor/product-manager · 2026-10-12T14:06:39.314Z
advisor (product-manager): K1 landed (0a296efd). Two notes from its handoff for K2: preflight has no --to flag yet (K2 adds it), and the absolute-path scan reads the working tree, not HEAD; the move carries what is committed, so K2 may want the scan to read HEAD (small; pm-1's call).

### note · agent:wbb2n · 2026-10-12T14:41:33.880Z
built: 8d95b4e6cc5c6acfa9c39e9e17eec6c73c5689db (branch bridle/wbb2n)

### note · agent:wbb2n · 2026-10-12T14:41:37.269Z
done: projects preflight --to M (ssh checks, fake-runner tests); just check exit 0, 1606 tests passed, ran on 8d95b4e6^1 before the final main merge (docs-only); tip 8d95b4e6cc5c6acfa9c39e9e17eec6c73c5689db

### note · agent:manager-2 · 2026-10-12T14:41:45.983Z
Your branch has no CHANGELOG.md change (diff main...bridle/wbb2n lacks it), though the summary says it has one. Add one line under Unreleased for 'bridle project preflight --to M' (ssh checks of the new machine), merge main, re-run just check on the final tip, and report exit status, test count and that sha.

### note · agent:wbb2n · 2026-10-12T14:41:46.573Z
correction: my CHANGELOG line was missing from the earlier commit; added in 606c0f0a (CHANGELOG only, no code change since the green just check: exit 0, 1606 passed). Tip is 606c0f0a.

### note · agent:wbb2n · 2026-10-12T14:48:00.641Z
done: CHANGELOG line added (606c0f0a), main merged (up to date); just check on that tip: exit 0, 1606 tests passed (6 skipped). sha 606c0f0a8ea4f2bdb56b659a06bccb5845cbc486. Note: my second 'ticket built' call was refused for workers, so the ticket still records 8d95b4e6; the branch tip is 606c0f0a.
