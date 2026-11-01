---
id: br-4mrx
title: "Wake waiters stop carrying messages: a wake only signals, the agent fetches what is pending"
kind: research
created: 2026-11-01
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

The human, 2026-11-01 ~8:00 AM ET (to aide), verbatim:

"And while we're at it, I'd want to discuss the fix for how the waiting system works. It's clearly not working in my mind. Agents are constantly starting waiters with the wrong command, which discards the output, and then constantly restarting.

My feeling is that we need to change the way messages are delivered. I want to have a good plan, think about this, and be clear on a better solution, because we've changed how this works multiple times. We're still not, in my opinion, doing this right.

My thinking is that the waiter should stop delivering actual messages because this reality and this risk of people starting waiters wrong is just never ending. There's always this cleanup where the agent has to go see, "Oh, did I miss a message?" trying to see if anything was lost.

I want you to research this, give me a good suggestion here, and look for edge cases I'm missing. My thinking is that we should only deliver basic exit codes, a message, and maybe a short description in the waiter. Based on the message code, the exit code, and maybe that description, the agent should then run a command to say, "Send me everything that's pending."

That way, if the agent starts a waiter wrong, pending messages won't be delivered or lost, and those wakes won't be lost. Starting the waiter wrong, which will continue to happen, will not cause any negative effects. We need clear instructions on what they do when they start the waiter wrong.

I know that they're getting a message when they start it wrong because they're telling me about it, but we need to update that if needed. If the waiter started wrong and then they're going to restart a new waiter, they need to be informed that we need to handle the race condition where something was delivered in that time period.

[...] maybe if they start the waiter wrong, nothing should be lost anymore. If they start again and something has arrived, maybe the waiter exits immediately and says something has arrived, and then they check it and start it again. I don't know. I really want this race condition issue to be handled.

My other thought about an edge case or a concern is that some of these agents I'm shutting down overnight or for a few days, and they may be getting nudges, wakes, and messages during that time. They should be able to catch up, but I'm a little concerned that we might have a massive amount of messages or wakes that pile up. Consider that in this design as well."

## The ask

Research and a recommended design, for the human to decide on (kind research: waits for the human; nothing is built from this ticket). The aide is doing the first pass with the human directly at their request: the current design (docs/design/agent-host/messages.md, api.md, cli.md `agent wake`), the failure modes seen, a proposal where the waiter only signals (exit code + short reason) and a separate command fetches everything pending, and the edge cases: starting a waiter wrong, the race between waiters, catching up after an agent was off for days (pile-up), and what marks a message read. The outcome is a design the human signs off on, then a feature ticket for the build, linked `related`.
