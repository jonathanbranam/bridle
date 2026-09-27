# Goals and non-goals

## Goals

- **Send agents off and let them coordinate.** Dependencies, waiting, messages,
  questions — without the human or a lead agent relaying everything.
- **Parallel by default, serialised only by a real conflict**, and let agents
  find and negotiate those conflicts themselves.
- **One workflow, many projects**, installed and managed the same way everywhere.
- **Human attention spent on decisions and acceptance only.**

## Non-goals

- Fleet scale. This is one person with a manager agent and a handful of workers, not
  Gas Town's 20–30 agents or Wheelhouse's 12,000 commits a day. Every trade that
  Beads v1.0 made for throughput (Dolt, daemon, memory decay) is out of scope.
- Autonomous merge-to-production. Research 03 §4 is the reason.
- A general orchestrator for other people. It is opinionated for these six repos
  and this workflow; the layering exists so *this designer's* projects can
  differ, not so strangers can configure it.
