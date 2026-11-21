+++
id = "br-fq8z"
title = "Measure build and test times on the laptop: what makes a change-compile-test cycle take ~an hour"
kind = "research"
state = "pending"
created_at = "2026-11-21T15:08:46.991Z"
updated_at = "2026-11-21T15:08:46.998402Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
+++

submitted by external:aide@vesta

**Hold: talk to the human first.** The human, 2026-11-21 (via the notes aide): "file it but tell the aide to talk to me first to clarify what i want". Don't start the research until the human has clarified the ask with bridle's aide.

## The ask (draft, to be clarified with the human)

Measure on the laptop (dalek), and compare with the NUC (vesta):

- incremental compile, full build, and the test suite, split so it's clear which dominates;
- what makes the laptop unusable during it (CPU/RAM, parallel jobs, the link step);
- options to cut it (fewer jobs / codegen units, a faster linker, splitting crates, test selection, building on the NUC).

Research only: no code changes.

## Why (the human's words, 2026-11-20)

"The Rust solution is destroying my laptop, compiling, taking an hour to make a change, compile it, and run all the tests. With Flume, I'm making changes in minutes, and there's no compiling. Obviously, I don't have as many tests, so that's a big part of it, I'm sure, with Rust. I should have the Rust project do some research on how long testing is and how long compiling is, but definitely, compiling that binary just is destroying my laptop. It just puts it on its knees. I can barely use the laptop, and it seems to be compiling. I'm pretty sure not actually testing, but anyway."

Filed by aide (notes) from the human's notes (area/coding/Bridle and Flume.md).

## Thread

### note · external:aide@vesta · 2026-11-21T15:08:46.998Z
submitted by external:aide@vesta
