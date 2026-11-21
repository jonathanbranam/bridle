---
id: br-fq8z
title: "Measure build and test times on the laptop: what makes a change-compile-test cycle take ~an hour"
kind: research
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

## HELD: the human's aide talks to the human first

The human, 2026-11-21 (via the notes aide on vesta), verbatim: "file it but tell the aide to talk to me first to clarify what i want". Nobody plans or starts this until the human has clarified the ask with bridle's aide.

## The ask (draft, to be clarified with the human)

Measure on the laptop (dalek), and compare with the NUC (vesta):

- incremental compile, full build, and the test suite, split so it is clear which dominates;
- what makes the laptop unusable during it (CPU and RAM, parallel jobs, the link step);
- options to cut it (fewer jobs or codegen units, a faster linker, splitting crates, test selection, building on the NUC).

Research only: no code changes.

## Why (the human's words, 2026-11-20)

"The Rust solution is destroying my laptop, compiling, taking an hour to make a change, compile it, and run all the tests. With Flume, I'm making changes in minutes, and there's no compiling. Obviously, I don't have as many tests, so that's a big part of it, I'm sure, with Rust. I should have the Rust project do some research on how long testing is and how long compiling is, but definitely, compiling that binary just is destroying my laptop. It just puts it on its knees. I can barely use the laptop, and it seems to be compiling. I'm pretty sure not actually testing, but anyway."

Filed by the notes aide (aide@vesta) from the human's notes (area/coding/Bridle and Flume.md).

## For the aide's talk with the human (pm-1 notes, not decisions)

- Prior work exists: the resolved build-cost ticket (b7cz, "build cost on the laptop") and the workers' `check-affected` fast path (`just check-affected`). Read both first so the human is not asked what is already known.
- Questions that decide the shape: is the pain the human's own interactive builds, or the workers' builds and `just check` runs on dalek while the human works (the orchestrator's note of 2026-10-09: "easy changes still run builds and peg the CPU")? Is moving worker builds to vesta or remus acceptable? Is a smaller `just check` for workers acceptable?
- Probably the answer is several tickets (cheap measures first), not one research report. The PM splits it after the human answers.
