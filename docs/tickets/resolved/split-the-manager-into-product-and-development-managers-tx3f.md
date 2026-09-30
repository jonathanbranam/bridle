---
id: tx3f
title: Split the manager into a product manager and a development manager
opened: 2026-09-27
resolved: 2026-09-29
repos: [bridle]
changes: [89b560e]
specs: []
needs: []
see: [hj4g]
---

## The question

Today one `manager` role does everything between the human and the workers:
triage, planning and right-sizing tasks, spawning and briefing workers,
reviewing and merging ([[docs/design/roles-and-lifecycle|roles]]). The human
wants that split into two roles. What exactly does each decide, how do they
hand work to each other, and what does each need from bridle (task records,
`ready`, claims)?

The human's words, 2026-09-27:

> Manager should be in charge of ticket triage entirely. There should be an
> agent also working the backlog of tickets and preparing them for work.
> There should be a separate roles for preparing work and for overseeing
> work. A project or product manager and a development manager

and, on right-sizing, the same day:

> Worker tasks should be sized to be complet able within that window as much
> as possible. That is an important job that must be done - right-sizing
> tickets. That work should be governed by the manager before passing work
> off.

## Why it matters

The manager's context fills with two unrelated jobs, and preparing work
(reading the backlog, splitting and sizing tasks) competes with overseeing it
(watching workers, reviewing, merging). The orchestrator also ends up doing
triage the manager should own.

## Resolution

Resolved by 89b560e: a `product-manager` role (project-defined, `autostart = true` in bridle's own `.bridle/config.toml`, prompt `workflow/base/roles/product-manager.md`) owns the backlog, triage and right-sized task preparation; the `manager` runs the work. The answer lives in docs/design/roles-and-lifecycle.md.
