---
id: awh4
title: "Specs: step text can't quote code or markup"
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-awh4]
---

## The ask

bridle spec check rejects markup in step text (backticks, <code>, HTML), so a rendering spec cannot quote the raw markdown it renders or the HTML it expects. The meta-notes-ui worker rephrased steps and normalised HTML quotes in step code, making scenarios less exact. Wanted: a way to carry literal text in a step (a quoted string, or a docstring/table argument not parsed as markup). Check crates/bridle-spec and the spec-file format doc for where the markup rejection lives and pick the smallest form that fits the format. Source: meta-notes-ui spec 3 (mu-pfgp, design/specs/rendering.md), its ticket myeg (specs friction log). Verify: just check, plus a spec test that quotes backticks and HTML in a literal.
