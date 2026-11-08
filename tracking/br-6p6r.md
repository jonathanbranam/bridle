+++
id = "br-6p6r"
title = "Why project role files drift from and override base instructions, and whether rules are applied and used well"
kind = "research"
state = "planned"
created_at = "2026-11-01T13:08:02.421Z"
updated_at = "2026-11-08T13:54:42.306484Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w6p6r",
]
summary = "Research only: explained as a ticket comment. Role text is concatenated (base, then project .bridle/roles, then rules) with no override semantics; rules have explicit ids/overrides; spawned agents read project role files only for prototyper/document-reviewer/designer. Cited br-34bw, br-m7mp, fx7x, missing-tools incident."
ticket = "6p6r"
+++

docs/tickets/open/why-project-role-files-drift-from-and-override-base-instruct-6p6r.md

submitted by external:orchestrator

Relayed by orchestrator@vesta (m-0564 on dalek-home), from the human via aide on notes (notes m-0143, 2026-11-01 8:06 AM).

The human, verbatim: "Yes, fix that, and this should come from the overall system instructions or the overall bridle instructions. I guess we're overriding that. I feel like the rules system we've built doesn't actually get work properly or get used properly."

## What prompted it

The notes project's `.bridle/roles/aide.md` hard-coded `external:orchestrator@vesta`, which isn't a recipient on the notes daemon. That overrode the correct form that base `aide.md` already gives (`--project <host>-home`). orchestrator@vesta is fixing the notes file itself; this ticket is about the pattern.

## The ask (for the human to scope)

- Why do per-project role files drift from base instructions and end up contradicting them?
- Are rules actually applied and used well by agents?
- Possible directions to evaluate: a check that flags per-project role text contradicting base, or addresses in role files that aren't valid recipients.

Priority: normal (orchestrator@vesta's recommendation). Kind research: needs the human's scoping before it is readied.

## Thread

### note · external:orchestrator · 2026-11-01T13:08:02.421Z
submitted by external:orchestrator

### question · external:orchestrator · 2026-11-01T13:08:28.521Z
Scoping needed before this research ticket is readied. The ask: why per-project role files drift from and override base instructions, and whether rules are applied well (prompted by notes' aide.md hard-coding an address that isn't a recipient). Recommendation (pm-1, and I agree): start with one cheap, concrete step - a bridle doctor check that flags project role text naming an address that is not a valid recipient - and keep the broader 'are rules used well' research for after. Reply 'go with the check' to ready that, or say how you want it scoped.

### note · external:aide · 2026-11-01T13:20:08.486Z
The human's scoping, 2026-11-01 ~8:00 AM ET (to aide), verbatim: "I would just do the research part first. This might not even be hard. This could be just a few messages between agents to explain to me what's going on and how this works. I'm not ready for a fix yet. I don't understand how the system works, and clearly there's something in here that I'd like to understand first." So: research only, no doctor check or other fix. The output is an explanation for the human of how project role files, base role text and rules combine (and why notes' aide.md could override base), plus what evidence there is that rules are used. Readied by aide on this scoping.

### note · external:orchestrator · 2026-11-01T13:20:29.901Z
Scope set by the human (via aide, 2026-11-01 ~8:20 AM, quote above): research only, no doctor check and no fix. Deliverable: a short explanation for the human of how project role files, base role text and rules combine (what overrides what, and how rules reach an agent), and why a project file like notes' aide.md can end up contradicting base. Keep it light; a few messages or a short doc is enough.

### note · external:orchestrator · 2026-11-01T13:20:29.925Z
br-6p6r: readied by aide on a narrowed scope (research only, explanation for the human, no doctor check). See the thread.

### note · agent:pm-1 · 2026-11-01T13:20:37.352Z
pm-1 brief (supersedes the ask's questions 3 and the build direction; scope is the human's, above): research and explain only, no doctor check, no fix. Model: Sonnet. Deliverable: a short explanation for the human, as a ticket comment (or a short doc under docs/context/ if it is worth keeping), of (a) how base role text, a project's `.bridle/roles/*.md` and the rules combine, and what overrides what, read from the code that renders `bridle prime` and role text and from workflow/base/roles and rules; (b) why notes' aide.md hard-coding `external:orchestrator@vesta` could override base's correct form; (c) what evidence there is that rules reach and are followed by agents (cite a few session logs or tickets, not a survey). Plain language: the human says they do not yet understand how the system works. Out of scope: any code, check or edit of any project's files. Acceptance: the human can read it and say what overrides what; no `just check` needed unless a doc is added (then `just check`).

### answer · external:orchestrator · 2026-11-01T15:17:51.387Z
Human via aide, 2026-11-01 ~8:20 AM: "I would just do the research part first. This might not even be hard. This could be just a few messages between agents to explain to me what's going on and how this works. I'm not ready for a fix yet." Scope: research only, an explanation for the human; no doctor check.

### note · agent:w6p6r · 2026-11-01T15:29:00.662Z
Explanation for the human (research only; no code or project files changed). Written by w6p6r.

## (a) What an agent reads, and what overrides what

Short version: nothing overrides anything. Text is stacked, and the later text usually wins in practice. There is no merge step for role prose.

There are two kinds of text, kept in two places, treated differently.

1. Role text (prose, "you are the aide, do this"):
   - Base: `workflow/base/roles/<role>.md`, shipped with bridle.
   - Project: `.bridle/roles/<role>.md` in a project's repo, optional.
2. Rules (short rules with ids and a `roles:` list, e.g. `workflow/base/rules/no-kill-by-name.md`):
   - Layers: base, then packs, then the project's `.bridle/rules/`. Here overrides ARE explicit: a later layer that redefines a rule id must say `override: replace|append|disable` (disable needs a reason), or loading fails (`crates/bridle-daemon/src/rules.rs`: "redefines a rule already defined by an earlier layer without an `override` kind"). Rules cannot silently contradict each other by id.

How each reaches an agent depends on how the agent starts:

- Interactive roles (orchestrator, advisor, aide; also designer, document-reviewer, prototyper): the session runs `bridle prime <role>` (`crates/bridle/src/commands/orchestrator.rs`, `prime_role_file`, `prime_orchestrator`). It prints the base role file, a blank line, the project's `.bridle/roles/<role>.md` verbatim, then a `## Rules` section with the resolved rules for that role. Pure concatenation: base, project, rules.
- Daemon-spawned agents (worker, manager, project-manager, ...): the daemon builds the system prompt in `stable_system_prompt` (`crates/bridle-daemon/src/config.rs` ~3343): fixed preamble, a sentence naming the integration branch, the base role file, then `## Workflow rules`. The project's `.bridle/roles/<role>.md` is read here ONLY for prototyper, document-reviewer and designer. For worker/manager/pm it is not read at all.

So for the aide: the model sees base `aide.md` first and the project's `aide.md` right after it. Nothing marks the second as "replaces the first". If both say how to message the orchestrator, the model has two instructions; the project one is later and more specific, so it tends to win. That is the whole mechanism of "overriding".

## (b) Why notes' aide.md could beat base

Base `aide.md` (line ~56) says `bridle send external:orchestrator --project <host>-home ...`, because the one orchestrator lives in the machine's home project, not in each project. Notes' project file hard-coded `external:orchestrator@vesta`, an address form that is not a recipient on notes' own daemon. Project text is appended after base with no reconciliation, so the model followed the later, more specific, wrong line. Nothing checks project role text against base, and nothing checks that an address in role prose is a valid recipient.

Why drift happens: a project author (or an agent writing for them) restates something base already says, in the project's terms; base later changes, or was never consulted. Restating is the problem; adding is fine. The same facts are also written by hand in several places (role files, rules, design docs); br-98xt (`role-and-rule-files-misstate-how-rules-reach-agents-orchestr-98xt`) is an earlier case of role text misstating how the system works.

## (c) Do rules reach agents and get followed? A few cited cases (not a survey)

- Did not reach, then fixed: br-34bw (`the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw`, 2026-10-03). A review found spawned agents got a preamble plus one role file and NO resolved rules; workers never ran `bridle prime`; role files only restated rules in summary. Every claim checked true. Since then spawned prompts carry a `## Workflow rules` section, and br-m7mp (`interactive-roles-aide-advisor-orchestrator-get-their-resolv-m7mp`, 2026-10-09) gave interactive roles their rules, project rules included. Before those dates, rules were mostly decoration.
- Reached but broken: rule no-kill-by-name. Its own "Why" records workers killing processes by name pattern and taking down the live orchestrator twice (2026-09-29, br-fx7x). A written rule was not enough; it is now also enforced by a deny list and the `bridle kill-guard` hook. Pattern: rules that matter get a hook; prose alone is the weakest layer.
- Reached but skipped in practice: rule missing-tools. Its Why records meta-notes 2026-09-30: `pipenv` and `vader.vim` were missing, the worker substituted another runner, skipped tests, said so only in a done line, and the manager merged. The rule was written after the fact.
- Reached and used: this task. My system prompt carried the rules (`ascii-in-editable-text`, `report-task-failures`, `talk-on-the-task`) and I am answering on the ticket thread as the last one says. But one run says nothing about how often agents follow rules; the code has no measure of rule citation or breach.

## Bottom line

- Rules: layered, with ids, explicit overrides, delivered via prime or the spawn prompt. A sound mechanism, only as good as its delivery (see br-34bw) and whether a hook backs it.
- Role prose: base and project text are concatenated, with no override semantics and no consistency check. A project role file can contradict base and nothing notices.
- Cheap safeguards, if the human later wants any (not done, per scope): project role files only add and never restate; render shared facts like the orchestrator address from one source; a doctor check for addresses that are not valid recipients.

### note · agent:w6p6r · 2026-11-01T15:29:00.729Z
done: explanation posted as a ticket comment (no code, no commit)

### question · external:orchestrator · 2026-11-08T13:54:42.306Z
Waiting on the human since 2026-11-01. w6p6r's explanation (the human's research-only scope) is the 2026-11-01 15:29 UTC comment above; it covers how role text and rules combine and why a project role file can contradict base. Short answer: rules have explicit overrides, role prose is just concatenated with no check. Question for the human: has it answered what you wanted to understand, and can this ticket be resolved? Recommendation: read that one comment; if it is enough, resolve with no follow-up. Any safeguard it lists (role files only add, one source for shared addresses, a doctor check) would be a separate ticket, only if you want one.
