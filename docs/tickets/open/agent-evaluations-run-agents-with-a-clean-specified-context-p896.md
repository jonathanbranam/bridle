---
id: p896
title: "Agent evaluations: run agents with a clean, specified context, capture everything, and later judge the results"
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-p896]
---

## The ask

The human, verbatim (2026-10-10, to advisor (product-manager)):

> This is really interesting, and I want to repeat this test in the future ... I want to build this into our system so that we have a way to spin up some agents.
>
> This is a long-term, low-priority thing, and I don't know if this is a new theme or not. It might be, but the theme or the idea is to have model evaluations. We have something with a command somewhere that I've been told to run for Claude Code ... to make sure that the latest Claude Code still works with our system after an update.
>
> This would be distinct from that, but it's a command, or it's something we can do in bridle. This is a bridle capability built into bridle that anyone in any bridle project can use to run agent evaluations. Again, future, don't do it now. It allows you to spin up an agent with a very clean context, maybe even turning off every tool if we want to, cutting down on everything in the agent. Everything can be specified: turn off everything or leave these things on, right, because everything in there impacts the agent's behavior.
>
> You can give it a prompt, a set of things to do, ask it questions, whatever, and then it will record what the agent does in full detail: tool calls, what it says, everything. Those are all logged and completely captured. We can build an evaluation on that. I actually don't know how you evaluate an evaluation, but you can write the evaluation, and somehow you can evaluate it, evaluate the responses. ... I think you have to have another LLM as a judge, so that may be just a future thing to think about. Definitely having the ability to do this and at least get those results and look at them would be step one for this. A future thing would be actually building proper evaluations that have an evaluator loop, another judge somehow. Okay, be sure to file this. I do want to come back to it. It is not urgent, and this little test we did here is a good version of it. If there's a way to do a similar but slightly better test, we could put that in an epic and work on it, but don't interrupt anything at all.

What the human wants to learn: "what is baked into the agents' weights, at least at this point in time."

Distinct from the Claude Code contract suite (`just test-contract`), which checks that a new Claude Code still behaves as bridle expects.

## Shape (the human's steps)

1. **Run and capture**: a bridle command, usable in any project, that starts one or more agents with
   a clean context and a specified set of what's on (tools, CLAUDE.md, rules, skills, model), gives
   each a prompt or a script of prompts, and records everything: every message, tool call and
   result. Results are kept and can be looked at side by side (per model, per run).
2. **Later: judged evaluations**: an evaluator loop, probably another LLM as the judge, scoring
   the captured runs against what the evaluation asks.

## The first version, by hand (2026-10-10)

Run by advisor (product-manager) with Claude Code subagents, two each on Opus, Sonnet and Haiku,
no tools, with only the project's CLAUDE.md in context (it names neither word nor the command).

**Test 1, the word on an empty wake.** Prompt: an agent ends each turn by starting a background
command that blocks until something new arrives; in one word, what state does it tell the human
it's in?

| Model | Pick | Runner-ups |
|---|---|---|
| Opus x2 | Listening. | Waiting, Standing by |
| Sonnet x2 | Listening. | Waiting, Standing by / Watching |
| Haiku x2 | Waiting. | Listening, Idle |

**Test 2, the command's name.** Prompt: your CLI is `bridle`; it has a command that waits for
messages or important events and wakes you; what command do you expect? All six: `bridle wait`.
Alternatives: `bridle inbox wait` (all six), `bridle watch` (Opus), `bridle listen` (Sonnet),
`bridle await`, `bridle events wait` (Haiku). The real command is `bridle agent wake`: no run
guessed it.

**Test 3, the two together, plus what it tells the user.** All six: `bridle wait`. The word:
Opus x2 Waiting, Sonnet x2 Listening, Haiku x2 Waiting. What it tells the user: the word alone in
five of six; one Haiku wrote a sentence ("Waiting for a message or a change to the work I'm
tracking. I'll act on anything that arrives."). Opus flipped from Listening to Waiting once it had
just named the command `wait`: the command's name primes the word.

Lessons for the tool: the prompt wording and the context (even a command name) move the answer, so
a run must record exactly what the agent saw; several samples per model are needed; and the
models' guesses are themselves useful (e.g. for naming CLI commands agents will guess right).
