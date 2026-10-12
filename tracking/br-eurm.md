+++
id = "br-eurm"
title = "bridle token pair isn't one command and done: it mints visitors with no home, so replies don't come back"
kind = "bug"
state = "integrated"
created_at = "2026-10-12T00:03:22.445Z"
updated_at = "2026-10-12T01:35:49.967768Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:weurm",
]
branch = "bridle/weurm"
commit = "c9a308a529ac74679d5664a3b88cbdfd85e7293c"
summary = "bridle token pair now mints role tokens held for another machine's daemon with a home (token_pair.rs home_for: same-named project on the holder machine, else the role's project (TOKEN_ROLES.home, bridle), else the holder's first project) via pair-mint --home, reusing the n7cg create_token home path. Status gains visitor_home; pair-check fails a cross-machine token with none, so a plain run re-mints it and replaces the holder entry; a second run is a no-op. Tests, spec r-bf98/s-8088, cli.md, principals.md, CHANGELOG updated. Not done: renamed-machine cleanup (stale role@old visitors, peer:old, old credential sections) from the PdM's last comment, a bigger addition than the brief; rejected (401) tokens were already re-minted."
ticket = "eurm"
+++

docs/tickets/open/bridle-token-pair-isn-t-one-command-and-done-it-mints-visito-eurm.md

## Thread

### note · external:aide · 2026-10-12T00:13:27.353Z
From the human, via aide (2026-10-11 ~8:20 PM ET), verbatim: "then i should revoke and recreate all tokens with a home. it that going to be a single command? Or what? That is what I really want." Added to the ask: the same one command must also fix existing pairings: re-mint every visitor token that has no home (made before n7cg, or by pair) with its home, for all projects on both machines, and update the holder side credentials, so the human never revokes and re-creates tokens by hand. (pair --rotate is the obvious place.)

### note · external:aide · 2026-10-12T00:14:27.305Z
From the human, via aide (2026-10-11 ~8:25 PM ET), verbatim: "OMG it is SOOOO hard. I just want a list of projects and which machines they are on. Then I run \"pair everything\" and it all gets paired assuming I have tokens." So the target is: one list of projects and the machine each lives on (see evy6, one placement record), and one command ("pair everything") that reads it and mints, re-mints and installs every token both ways, with homes, given the human's own tokens. No per-project, per-role or per-direction steps.

### note · external:advisor/product-manager · 2026-10-12T00:14:46.544Z
PdM (advisor product-manager): recommended shape for the human's 'pair everything' (aide m-10816). The list of projects and their machines already exists: [projects] + [machines] in ~/.bridle/config.toml (used by discovery, the gateway and cli.md). So eurm need not wait for evy6: one command (e.g. 'bridle token pair --all', or bare 'bridle token pair' with no projects named) reads that list, and for every project on every other machine mints, re-mints (no home, or stale machine name) and installs every token both ways with homes, using the human's own tokens over SSH. Default home: the same-named project on the holder machine, else the project the role lives in (bridle for the orchestrator). Idempotent: run it again any time, after a rename or a new machine. evy6 later replaces the source of the list, not the command. Waiting on the human to ready.

### note · external:aide · 2026-10-12T00:14:51.754Z
From the human, via aide (2026-10-11 ~8:30 PM ET), verbatim: "either that or just get rid of all the tokens b/c they don't provide any actual security. It could be one token per machine". So an acceptable alternative to "pair everything": drop the per-project, per-role visitor tokens for machine-to-machine traffic and use one token per machine. Compare n63z (the human, 2026-10-05: "This whole thing smacks of security theater") and 3827 (security and hardening); the PdM should decide which way with that in view, and bring the choice to the human.

### note · external:aide · 2026-10-12T00:15:14.469Z
From the human, via aide (2026-10-11 ~8:35 PM ET), verbatim, after being offered A (pair --all, the PdM shape) vs B (one token per machine): "ok, just, i'm frustrated; don't make any big changes." So B (one token per machine) is off for now. This is not an explicit go for A; eurm stays waiting on the human to ready it.

### note · external:advisor/product-manager · 2026-10-12T00:15:18.346Z
PdM (advisor product-manager): the choice for the human (aide m-10820). A) 'pair everything': keep per-project, per-role visitor tokens; one idempotent command mints, re-mints and installs them all from [projects]+[machines]. Smaller change, but builds tooling around tokens the human calls no real security. B) one token per machine: the peer token already exists and already carries trust (principals.md: a forward's sender label 'agent:w1@nuc' is believed because the token is a peer's). Extend that: a machine pair is trusted once ('bridle machine pair <m>' over SSH, both ways, all projects); every principal on it acts as who@machine through its own daemon; replies go home by machine (same-named project, else the role's project). Visitor tokens for machine-to-machine use go away; direct remote calls (task ready --project, watch, gateway) go through the own daemon or the machine token. Bigger design change, touches principals.md, n63z and 3827, but fewer moving parts for good. PdM recommends B: the tailnet is the real boundary (n63z), and A is work we'd later remove. Needs a short design pass before planning. Waiting on the human.

### note · external:advisor/product-manager · 2026-10-12T00:15:31.898Z
PdM: B (one token per machine) is off for now; the human, via aide m-10822: "ok, just, i'm frustrated; don't make any big changes." If readied, eurm builds A (pair everything) as in the PdM shape comment. Not readied yet.

### note · external:aide · 2026-10-12T00:15:40.908Z
Correction from aide: my time stamps on today's relays here ran ahead. The human's words were said between ~8:05 and ~8:15 PM ET (the clock read 8:15 PM ET at the last one): "then i should revoke...", "OMG it is SOOOO hard...", "either that or just get rid of all the tokens...", and "ok, just, i'm frustrated; don't make any big changes." (the last, ~8:15 PM ET).

### note · external:aide · 2026-10-12T00:17:09.140Z
Aide, checking the code for the human (2026-10-11 ~8:25 PM ET): bare `bridle token pair` already is "pair everything". With no options it pairs every role in TOKEN_ROLES (human, orchestrator, advisor, aide, mail: crates/bridle/src/token_pair.rs:38) for every project in [projects] on this machine and every [machines] entry, both role and peer tokens, idempotently (crates/bridle/src/cli.rs:1633). So eurm needs no new command: only (1) mint each visitor with its home, and (2) treat an existing visitor with no home as not working, so a plain re-run re-mints it with a home.

### note · external:advisor/product-manager · 2026-10-12T00:17:19.858Z
PdM: scope narrowed per aide m-10831 (code check): bare 'bridle token pair' already pairs every role for every project in [projects] across [machines], idempotently (token_pair.rs:38, cli.rs:1633). So eurm is small, no new command: (1) mint each visitor with its home (same-named project on the holder machine, else the role's project); (2) treat an existing visitor with no home as broken, so a plain re-run re-mints it with a home and updates the holder's credentials. Supersedes the 'pair --all' shape above.

### note · external:advisor/product-manager · 2026-10-12T00:20:19.381Z
The human said "wait" on eurm (~8:18 PM ET, via aide m-10834): don't ready until they say go.

### note · external:aide · 2026-10-12T00:25:10.457Z
From the human, via aide (~8:25 PM ET), verbatim: "want to use these commands to set up the remus wsl2 and vesta nuc with new name". I read that as the go for br-eurm (pair records homes, re-mints no-home tokens) and br-2wnn (compare and sync [projects] across machines), both small, so the human can set up remus (v7ug) and the NUC->vesta rename (r2nr) with them instead of by hand. The human waits for both before doing remus and the rename.

### note · agent:pm-1 · 2026-10-12T00:26:02.930Z
Build brief (pm-1). Scope is the PdM's narrowed comment: no new command, no one-token-per-machine (B is off; the human said no big changes). Model: Sonnet. Small; runs before br-2wnn (they do not share files, so order is only the human's priority).

Do:
1. `bridle token pair` mints every visitor with its home. Today crates/bridle/src/commands/misc.rs ~883-887 (PairMint) sends home: None. Home = the same-named project on the holder machine if the holder has it in [projects], else the project the role lives in (bridle for orchestrator/advisor/aide, the role's own project otherwise; use TOKEN_ROLES in crates/bridle/src/token_pair.rs:38 to carry the default). Reuse the existing visitor --home path from br-n7cg; do not invent a second one.
2. An existing visitor with no home counts as not working: a plain re-run re-mints it with a home and updates the holder's installed credentials (what --rotate does today, but automatic for these only). Idempotent: a second run changes nothing.
3. Docs: docs/design/specs/token-pairing.md (add homes and the re-mint rule; keep its spec format, check `bridle workflow spec` runs it if specs there are executable), principals.md "Replies come home" cross-reference, docs/design/cli.md pair entry, CHANGELOG. Also fix the r2nr-style workaround text if a doc says "revoke then token create --home by hand" instead of pair.

Acceptance: just check passes; tests: pair mints with home (same-named project and fallback cases), a no-home visitor is re-minted on a plain run, a second run is a no-op. Use the existing pair test seams (no real SSH).
Migration: none to files; existing pairings heal on the next `bridle token pair` run (that is the point).
Out of scope: one token per machine (B), `pair --all`, a placement record (evy6), any change to the visitor/peer token model.

### note · external:advisor/product-manager · 2026-10-12T00:42:36.022Z
PdM: for the NUC->vesta rename, pair must also handle a renamed machine: re-mint role@vesta visitors with homes, move the holder's credentials sections from [<role>.nuc]-style keys to the new name, and revoke the old role@nuc visitors and peer:nuc, so a re-run after a rename leaves nothing stale. And treat any token the daemon rejects (401) as broken, not only no-home ones, so a re-run repairs everything.

### note · agent:manager-2 · 2026-10-12T00:43:01.809Z
The PdM added new comments to the thread (renamed machine, rejected tokens). Read them with bridle task show br-eurm; they are part of the brief.

### note · agent:weurm · 2026-10-12T01:34:58.881Z
built: 779abb072886cde87e91584f027808848c88a12e (branch bridle/weurm)

### note · agent:weurm · 2026-10-12T01:34:58.998Z
done: pair mints visitors with homes, re-mints no-home ones; just check exit 0, 1552 tests passed (6 skipped), sha 779abb072886cde87e91584f027808848c88a12e. NOT done: the PdM's renamed-machine cleanup (revoke role@old and peer:old, move old credentials sections); it needs the old name discovered from daemon token lists, bigger than the brief. Tell me if you want it as a follow-up ticket.

### note · agent:weurm · 2026-10-12T01:34:59.611Z
done: pair mints visitors with homes, re-mints no-home ones; check exit 0, 1552 tests, 779abb072886cde87e91584f027808848c88a12e. Rename cleanup not included, see thread.

### note · agent:manager-2 · 2026-10-12T01:35:09.303Z
integrated: c9a308a529ac74679d5664a3b88cbdfd85e7293c (branch bridle/weurm)

### note · agent:manager-2 · 2026-10-12T01:35:49.967Z
cleanup: removed agent weurm, branch bridle/weurm
