# Goals: direction, not scope

Goals are the long-term direction. They are **not a backlog**: most of them are
not being built now, and some are deliberately left out of the current design
because building toward them now would be too complex. Each goal is
categorised on two axes, plus a stance that says how the current design relates
to it:

```markdown
## Offline-first clients                         {#g-03}
firmness: firm · priority: later · stance: unaddressed

Every client should work without a network connection and sync on reconnect.

**Why unaddressed:** it needs a sync engine and a conflict model we don't have.
Designing for it now would stall everything else. The current design is
online-only on purpose.
```

| Axis | Values | Meaning |
|---|---|---|
| **firmness**: how locked the requirement is | `fixed` · `firm` · `soft` · `open` | `fixed`: the project will meet this as stated. `firm`: intended, but the shape may change. `soft`: likely, details open. `open`: a direction we are curious about, which may be dropped |
| **priority**: when it matters | `now` · `next` · `later` · `someday` | ordering of attention, not a schedule |
| **stance**: how the current design relates | `build` · `keep-open` · `unaddressed` | `build`: current work may implement it. `keep-open`: don't implement it, but don't make it harder. `unaddressed`: the current design intentionally ignores it |

The stance defaults from priority (`now`→`build`, `next`→`keep-open`,
`later`/`someday`→`unaddressed`) and can be set explicitly. `unaddressed` needs a
one-line *why*.

Base rules for goals (locked):

- **A worker never implements a goal its task does not link to.** Goals are
  context, not instructions.
- **A gap between a goal and the current design is expected.** Agents don't
  report it unless the goal's stance is `build`. An `unaddressed` goal missing
  from the design is the intended state.
- **Plans cite the goals they serve** (`serves: g-03`), so the manager can see
  which goals have work behind them.
- **Changing a goal's firmness, priority or stance is the human's call.** An
  agent can propose a change with `bridle goals propose`.
