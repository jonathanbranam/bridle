---
id: br-jy8n
title: "Mentions need a stricter syntax: a bare @word in ticket text shouldn't be treated as a mention"
kind: feature
created: 2026-11-19
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-11-18, via flume's advisor (m-19141), verbatim:

> "we should be able to write the @ symbol inside of a document without it becoming a comment, I think, or whatever it is. There should be a specific syntax required that's a little bit more restrictive than just that for sending a message to somebody, to an agent."

What happened: a comment on fl-mv4r (flume's workflow layering design) mentioned the heading tags `@add`, `@remove` and `@replace`. The daemon (`crates/bridle-daemon/src/mentions.rs`) took them for mentions and answered "no such recipient for @add, @remove, @replace; nobody was told (write \@name for plain text)".

The ask: a mention needs a stricter syntax than a bare `@word`, so `@` can appear in ticket text and comments without escaping. Choosing that syntax is a design decision for the human: for example, only `@` followed by a known principal, or an explicit form such as `@agent:name` or `@[name]`. An unknown `@word` should stay plain text instead of raising an error.
