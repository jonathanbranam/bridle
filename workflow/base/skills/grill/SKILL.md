---
name: bridle-grill
description: Grill the human, one or two questions at a time, to clarify a bridle ticket or epic until the understanding is shared, and check a child ticket against its parent epic. Use when a write-up is vague, an epic needs an outcome and design, a ticket may contradict its epic, or the human says "grill".
---

# bridle-grill

Adapted from the `grilling` skill in Matt Pocock's skills
(https://github.com/mattpocock/skills, `skills/productivity/grilling/SKILL.md`,
commit 95249b0b), used under the MIT license below. Changes: one or two
questions at a time, bridle tickets as the record, a session-mode question and an epic-or-ticket question first, ASCII only.

> MIT License. Copyright (c) 2026 Matt Pocock
>
> Permission is hereby granted, free of charge, to any person obtaining a copy
> of this software and associated documentation files (the "Software"), to deal
> in the Software without restriction, including without limitation the rights
> to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
> copies of the Software, and to permit persons to whom the Software is
> furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
> FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
> AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
> LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
> SOFTWARE.

Interview the human relentlessly until you reach a shared understanding of one
bridle ticket. Map it as a **design tree**: every decision branches into the
decisions that hang off it. You never ready, plan or build.

## Start

1. Read the ticket (`bridle ticket show <id>`) and what it names: code, docs,
   specs. For a child ticket (it has a `parent`), read the parent epic first:
   its outcome, done-when and design.
2. Ask first: **"Are you going to use (1) mobile with keyboard, (2) computer
   with keyboard, or (3) voice for this session?"** The answer sets the pace
   (see Asking). Then ask: **"Is this an epic or a ticket?"** (give your reading of it as the
   recommended answer). An epic needs an outcome, a done-when and a design, plus
   a list of what is not yet specified, so its children can be checked against
   it. A ticket needs a clear ask and a way to verify it.

## Asking

Work the tree **one or two questions at a time**. Too many at once overwhelms
the human and stalls the work. The session mode sets the limit:

- **Computer**: one or two questions a round.
- **Mobile**: one or two questions a round, each kept short.
- **Voice**: exactly one question a round, a sentence or two, the
  recommendation in one line, no tables or long lists.

 Ask the questions whose prerequisites are
settled, pick the one or two that matter most, and wait.

Format each question like so:

```
**Q1 (N open)** - <question title>: <question body, may include choices>

Recommended: <your answer>
```

Word each question so "yes" accepts your recommended answer. `N` is how many
questions are open now, including this one. It can grow as answers open new
branches; say so when it does. Keep the open questions written on the ticket
(in its body or a comment), so the human can see the rest and does not take
the batch in front of them for everything that is left.

For a child ticket, each gap or contradiction with the epic is a question:
"the epic says X, this ticket assumes Y: which?" Do not resolve it, and do not
edit the epic, without the human's answer.

## Facts and decisions

Finding _facts_ is your job, never the human's. When a question needs a fact
from the code, docs or tickets, look it up (a sub-agent for anything large);
do not ask. A running lookup is an unsettled prerequisite: only the questions
downstream of it wait. The _decisions_ are the human's: put each to them and
wait; never answer one yourself.

## Record

The ticket is the record (rule `talk-on-the-task`); chat alone is not.

- After each answer, `bridle ticket comment <id>` with the question and the
  human's words quoted.
- Put the settled result into the ticket body as it settles: the ask, and for an
  epic the outcome, done-when and design.
- If the ticket holds several things, say so and propose a split. Filing the
  children is the project manager's call.

## Done

The session is done when no question is left open and nothing is silently
assumed. Summarise what is settled and what is still unspecified, then ask:
**"Shared understanding?"** Stop on the human's yes. Do not ready, plan or
build the ticket.
