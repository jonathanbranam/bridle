---
id: d9wq
title: "Research: product manager roles, human and agentic (BMAD and others), and what to call roadmap groupings"
kind: research
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [6h65, 95mu, 7r2c, ukpm]
tasks: [br-d9wq]
---

## The ask

The human, 2026-10-09 ~9:00 AM ET, verbatim (to advisor product-manager, during the PdM trial,
docs/notes/product-manager-trial.md):

> also tbd - what do we call these? IDK about themes, workstreams feels nicely generic; are
> there other comon PdM terms? also, related, spin up a subagent to do web research on PdM
> roles. search for traditional human PdM role info as well as agentic PdM roles. then write a
> research ticket with the results; link to all references, include attribution; we can use
> this to guide our role. Check BMaD as well I'm sure he has a PdM agent.

Research done by a web-research subagent of advisor (product-manager) on 2026-10-09; results
below. Use it to guide the product manager role (6h65) and the roadmap's terms
(docs/notes/roadmap.md).

## Findings

Web research only. Every claim names its source as [Rn] (References below); quotes are short.
[UNVERIFIED] marks what couldn't be confirmed from a primary source.

### 1. Human product management

**The job**

- Marty Cagan (SVPG): four product risks (value, usability, feasibility, business viability). The
  PM owns value and viability and "is accountable for product outcomes"; the designer owns
  usability, the lead engineer feasibility. [R1]
- Cagan prefers outcomes to feature roadmaps: give teams "prioritized business objectives"
  (usually OKRs) and let them choose solutions, "It is all about outcome rather than output."
  Dated promises are kept apart as "high-integrity commitments", only "for those situations where
  we need to actually commit to a date or a specific deliverable." [R2]
- Melissa Perri, "Escaping the Build Trap" (2018): don't measure success by features shipped
  instead of customer outcomes; beware "project management culture" taking over product teams;
  the "product kata", an experiment loop around a problem. (Secondary summaries; book not
  read.) [R3]
- Teresa Torres: good teams "engage with customers at least weekly"; a product trio (PM,
  designer, engineer) runs interviews and assumption tests. [R4]

**Next to the other roles**

- Project manager: delivers one project "on-time, on-budget". Program manager: a "lateral view"
  across projects, coordinating dependencies. Product manager: what to build and why, over the
  product's life. Titles are often mixed in practice. [R5]
- Product owner (Scrum): "accountable for maximizing the value of the product", including the
  Product Backlog. [R6] Cagan: PO should be a role the product manager holds; splitting "the
  business person" from the one working with developers causes serious problems. [R7]
- Bridle mapping: the project-manager role (plans, sizes, owns the queue) is the project
  manager and Scrum PO's refinement job; the orchestrator is the program manager and delivery
  coordinator; the PdM owns the what and why and the outcome, upstream of the queue.

**Roadmaps**

- Now/Next/Later (Janna Bastow, ProdPad, first sketched 2012): time horizons, not dates;
  certainty falls from Now to Later; rank problems, not solutions. "The roadmap shows the plan.
  The OKRs carry the commitment." [R8]
- Theme-based roadmaps (ProductPlan): themes are "high-level objectives"; features stay in the
  backlog as the means to a theme. [R9]
- Roman Pichler's GO roadmap: date, name, goal, features, metrics. "I recommend using one
  product goal at a time"; three features per goal, never more than five. [R10]

**Discovery and delivery**

- Cagan, 2012: "The Discovery track is all about quickly generating validated product backlog
  items"; delivery is about "releasable software". He warns against "mini-waterfalls" of
  handoffs, and later dropped the "dual-track" name for continuous discovery and delivery. [R11]

**Gates**

- Stage-Gate (Robert Cooper): stages split by gates, where gatekeepers apply set criteria and
  decide Go, Kill, Hold or Recycle; many projects are expected to die at early gates. (Secondary
  source; check Cooper's own work.) [R12]
- Scrum: refinement makes items "ready for selection" (doable in one Sprint); the Definition of
  Done is "a formal description of the state of the Increment." The 2020 Guide has no formal
  Definition of Ready; that is community practice. [R6]
- SAFe portfolio Kanban: Funnel -> Reviewing -> Analyzing -> Ready -> Implementing -> Done. An
  Epic Owner moves each epic; Analyzing writes a lean business case (benefit hypothesis, MVP);
  Lean Portfolio Management decides Go/No-go (No-go goes to Done); the Funnel has no WIP limit.
  (Full article behind a login; from snippets and secondary pages.) [R13]

**Prioritising**

- RICE (Sean McBride, Intercom, 2018): Reach x Impact x Confidence / Effort, "total impact per
  time worked". [R14]
- WSJF (SAFe): Cost of Delay / job size, where Cost of Delay = user-business value + time
  criticality + risk reduction/opportunity enablement; small jobs rank higher; suits features and
  epics more than team backlogs. [R15]
- MoSCoW (Dai Clegg, 1994; DSDM): scoping a delivery window. [R16] Kano (Noriaki Kano, 1984):
  features by their effect on satisfaction. [R17]
- Opportunity solution tree (Torres): outcome -> opportunities -> solutions -> assumption
  tests. [R18]
- For bridle: RICE and WSJF need numbers agents would invent. Advisory only; the human owns the
  ranking.

### 2. What to call the groupings

| Term | Who uses it | Level |
|---|---|---|
| Theme | ProductPlan, Atlassian | A strategic objective or investment bucket. Atlassian's old Portfolio-for-Jira FAQ: themes "don't actually represent an extra level in the hierarchy"; they cut across initiatives. [R9][R19] |
| Initiative | Atlassian (Jira Plans), BMAD | Above epics, spanning quarters (Atlassian's example: cut launch costs 5% in a year). [R19] BMAD v6: "the business outcome its epics serve". [R20] |
| Epic | Scrum community, Atlassian, SAFe, BMAD | A large body of work split into stories; in SAFe, the top portfolio item. [R13][R19] |
| Capability / Feature | SAFe | Epic > capability > feature > story [UNVERIFIED, page behind login]. BMAD uses "capability ids" in specs. [R20] |
| Objective / Outcome / Goal | Cagan (OKRs), Pichler, Torres, Scrum (Product Goal) | The measured "why". [R2][R6][R10][R18] |
| Opportunity | Torres | A customer need or pain, below the outcome. [R18] |
| Bet / Pitch / Appetite | Shape Up (Ryan Singer, Basecamp) | See below. [R21][R22] |
| Track | wshobson "conductor" plugin; dual-track agile | In conductor, 1-5 days of work with spec.md and plan.md: far smaller than a roadmap grouping. [R23][R11] |
| Workstream, pillar | General business use | No framework defines them formally [UNVERIFIED as formal terms]. |

Shape Up: a pitch is shaped work, "rough, solved, bounded" [R22]; appetite is "The amount of time
we want to spend on a project, as opposed to an estimate"; at the betting table senior people
pick bets for a six-week cycle, with no backlog to groom ("Just a few good options to
consider"); a circuit breaker cancels work not shipped within its cycle by default. [R21]

The usual hierarchy: theme (a cross-cutting tag) > initiative > epic > story/task, with the
outcome or objective as the initiative's "why".

The researcher's recommendation for bridle:

- **Initiative** for the grouping the PdM owns: current agentic usage (BMAD v6's initiative
  template: Outcome, Done when, Boundaries, Notes, and an `after:` dependency list) [R20],
  matches Jira, and implies an outcome rather than a bag of tickets.
- **Theme** only as a cross-cutting tag, not a level.
- From Shape Up, **appetite** (the human sets a budget in agent-hours or dollars instead of
  asking agents for estimates) and **bet** (the human's decision to commit).
- Avoid **track** (clashes with dual-track agile and conductor's small units) and
  **workstream** (implies people or teams working in parallel, which means little with agents).

### 3. Agentic product manager roles

**BMAD Method** (bmad-code-org; latest release v6.12.1, 2026-10-04) [R24]

- John, the PM. His customize.toml: role "Translate product vision into a validated PRD, epics,
  and stories"; identity "Thinks like Marty Cagan and Teresa Torres. Writes with Bezos's
  six-pager discipline."; principles "PRDs emerge from user interviews, not template filling",
  "Ship the smallest thing that validates the assumption", "User value first; technical
  feasibility is a constraint". Menu: PRD (create, update, validate), CC (correct course
  mid-implementation), TK ("Slice initiatives, plan epics, and manage tickets"). [R25]
- Other agents: Mary (analyst: research, product brief), Winston (architect: architecture,
  check-implementation-readiness), Bob (scrum master: sprint planning, create-story), Sally (UX),
  Amelia (dev), Quinn (QA). Chain: Brief -> PRD -> Architecture -> Epics -> Story -> Code ->
  Tests. (Persona names from a third-party docs mirror.) [R26]
- Readiness gate: check-implementation-readiness confirms PRD, UX, architecture, epics and
  stories are complete and consistent before building. [R27]
- v6 planning paths route work by size (trivial, one session, epic, project) and by whether the
  intent is "well defined": "The spec skill writes the contract; it does not help you figure out
  what you want." The human reviews story order and picks which stories get a checkpoint;
  "foundational stories" get the human's attention "before automation repeats them";
  bmad-build-auto (unattended) only after "decisions stabilize"; a retrospective closes each
  epic. [R28]
- An initiative's notes record: Assumption, Open question, Unknown, Parked, Decision (dated, "so
  it is not asked again"), Source conflict. [R20]
- To borrow: routing by size and by how well the intent is defined; a dated decision log so the
  human isn't asked twice; human checkpoints on foundational work only; an explicit readiness
  check; "Done when" as an outcome, not "all children closed".
- Pitfall: BMAD is built for one interactive user switching personas in chat. Bridle's agents
  are headless and need a queue and gates that wait.

**MetaGPT** [R29][R30]: "Code = SOP(Team)"; product manager, architect, project manager and
engineer roles hand structured documents down an assembly line. The paper (Hong et al., 2023; v7
2024) aims to curb "cascading hallucinations caused by naively chaining LLMs". Borrow: structured,
checkable intermediate documents. Pitfall: no human in the loop; the PM turns a one-line
requirement into a PRD with no discovery.

**ChatDev** (Qian et al., 2023) [R31]: a waterfall "chat chain" (CEO, CPO, CTO decide the
design; programmers, reviewers, testers follow). Same pitfall: role-play in place of validation.

**GitHub Spec Kit** [R32][R33]: constitution -> specify -> plan -> tasks -> implement, plus
clarify, checklist, analyze and now converge; the human reviews after each step; "Define what
and why before deciding how". An idea assessment says go, needs-clarification or kill. Templates
require `[NEEDS CLARIFICATION: ...]` markers, and the spec checklist needs "No [NEEDS
CLARIFICATION] markers remain"; plan templates carry "Phase -1" gates (simplicity,
anti-abstraction). Borrow: clarification markers as a cheap, machine-checkable test of "defined
enough to build".

**Amazon Kiro** [R34]: requirements.md -> design.md -> tasks.md; requirements in EARS ("WHEN
[condition] THE SYSTEM SHALL ..."); feature specs pause for approval between phases, a "Quick
Spec" skips them; requirements-first or design-first. Borrow: gate strictness per item, so small
work skips gates.

**OpenSpec** (Fission-AI) [R35]: a folder per change, `openspec/changes/<name>/` (proposal,
specs as WHEN/THEN scenarios, design, tasks); the human reviews before `/opsx:apply`, and
`/opsx:archive` folds it into the main specs; favours "iteration over rigid phase gates". Borrow:
a change folder as the unit the PdM puts to a gate.

**Subagent collections**: VoltAgent's product-manager.md (model haiku) is a generic persona
with checklists like "User satisfaction > 80% achieved" and "Roadmap updated quarterly"; no gate
or handoff logic. Pitfall: personas that read like job ads produce generic output. [R36]
wshobson/agents has no PM agent that was found; its "conductor" plugin uses tracks with spec.md
and plan.md and a rule to "Review specs before planning". [R23]

**Factory and Devin** [R37]: Factory's "Product Droid" helps with product management and PRD
drafting (secondary sources; no approval gate verified). Devin has no product role; its human
checkpoint is the PR.

**HITL guidance**: Addy Osmani lists checkpoints (spec review, plan approval, code review, test
verification, deployment approval, escalation) and aims at "minimizing unnecessary human
intervention", focused on architecture, security and business logic. [R38] FeatBit (vendor):
gating every step breeds rubber-stamping; a gate needs context, approve/reject/narrow options
and an audit record. [R39]

### 4. Synthesis for bridle (the researcher's)

- **What the PdM owns**: everything not yet ready to build, from idea to ready (Cagan's value and
  viability; BMAD's "well-defined intent" judgement; SAFe's Funnel to Ready). It hands over at
  ready: the project manager sizes and queues, the orchestrator dispatches. After that it watches
  delivery against each initiative's outcome and "Done when", not ticket counts, and proposes a
  correction (BMAD's CC) when delivery shows the plan was wrong.
- **Artefacts**:
  1. Roadmap: Now/Next/Later of initiatives, no dates, each with an outcome line and an
     appetite; themes as tags; Pichler's one goal at a time limits Now.
  2. Initiative brief, after BMAD's template: Outcome, Done when, Boundaries, requirements with
     ids, Notes (assumptions, open questions, parked, dated decisions), and its ordered steps
     (design -> human review -> spec -> build) with `after:` dependencies.
  3. A readiness record per step: ready only when no `[NEEDS CLARIFICATION]` markers remain
     (Spec Kit), the human's decisions are recorded, and it fits the appetite; otherwise "needs
     the human" with the specific question.
  4. A decision log: dated decisions of the human, never asked again.
- **The human's touchpoints**, few, in Stage-Gate's Go/Kill/Hold/Recycle form:
  - Betting: the human takes an initiative into Now and sets its appetite (Shape Up). No
    grooming of a big backlog; parked pitches can come back.
  - Design or spec approval: before an initiative's first build step and for "foundational"
    steps (BMAD); later steps that repeat an approved pattern skip it (Kiro's Quick Spec).
  - Circuit breaker: when the appetite runs out, escalate to the human instead of extending.
  - An outcome review when an initiative closes (BMAD's retrospective).
- **Pitfalls**: persona-only PMs with no validation (MetaGPT, ChatDev, VoltAgent); invented RICE
  or WSJF numbers; gate fatigue; mini-waterfall handoffs (Cagan); duplicating the project
  manager's task sizing.
- **Terms**: "initiative" for the grouping, "theme" as a tag, "appetite" and "bet" from Shape Up.

## Advisor (product-manager) notes

- The trial's roadmap (docs/notes/roadmap.md) uses "workstream" for now. The research's case
  against it is that agents don't need parallel teams; its case for "initiative" is that each
  grouping should carry an outcome and a "Done when". The human decides; the roadmap will be
  renamed if the human picks "initiative".
- Cheap things the trial can try with today's tools: an outcome and "Done when" line per
  workstream; a dated decision log in the roadmap; "needs the human" items phrased as one
  specific question each (already done).

## References

- [R1] Marty Cagan, SVPG, "The Four Big Risks", 2017-12-04. https://www.svpg.com/four-big-risks/
- [R2] Marty Cagan, SVPG, "The Alternative to Roadmaps", 2015-09-07. https://www.svpg.com/the-alternative-to-roadmaps/
- [R3] Melissa Perri, "Escaping the Build Trap" (O'Reilly, 2018), via secondary summaries: https://productandrew.substack.com/p/product-41 and https://evansamek.substack.com/p/escaping-the-build-trap-by-melissa
- [R4] Teresa Torres, Product Talk, "Product Discovery", 2021-08-18. https://www.producttalk.org/2021/08/product-discovery/
- [R5] ProductPlan, "Product Management vs. Program Management". https://www.productplan.com/learn/product-management-vs-program-management ; PM Training, "Program Manager vs. Product Manager vs. Project Manager". https://blog.pmtraining.com/about/program-manager-vs-product-manager-vs-project-manager
- [R6] Ken Schwaber and Jeff Sutherland, The Scrum Guide, November 2020. https://scrumguides.org/scrum-guide.html
- [R7] Marty Cagan, SVPG, "Product Manager vs. Product Owner Revisited". https://svpg.com/product-manager-vs-product-owner-revisited/ ; Mind the Product, "Fireside chat with Marty Cagan". https://www.mindtheproduct.com/fireside-chat-with-marty-cagan/
- [R8] Janna Bastow, ProdPad, "I invented the Now-Next-Later roadmap...", 2022-10-18. https://www.prodpad.com/blog/invented-now-next-later-roadmap/
- [R9] ProductPlan, "Theme-based product roadmap". https://www.productplan.com/templates/theme-based-product-roadmap/ ; "Feature-less roadmap". https://www.productplan.com/glossary/feature-less-roadmap
- [R10] Roman Pichler, "The GO Product Roadmap", 2013, updated 2026-01-12. https://www.romanpichler.com/blog/goal-oriented-agile-product-roadmap/
- [R11] Marty Cagan, SVPG, "Dual-Track Agile", 2012-09-17. https://www.svpg.com/dual-track-agile/
- [R12] Stage-Gate (Robert G. Cooper), secondary: https://agilebrandguide.com/wiki/models/stage-gate-process/
- [R13] Scaled Agile, "Portfolio Kanban" (behind login). https://framework.scaledagile.com/portfolio-kanban/ ; https://scaledagileframework.com/portfolio-kanban ; Atlassian, SAFe lean business case template. https://www.atlassian.com/software/confluence/templates/safe-lean-business-case
- [R14] Sean McBride, Intercom, "RICE: Simple prioritization for product managers", 2018-01-05. https://www.intercom.com/blog/rice-simple-prioritization-for-product-managers/
- [R15] Scaled Agile, WSJF (archived). https://webarchive.library.unt.edu/web/20161225132214mp_/http://www.scaledagileframework.com/wsjf ; ProductPlan glossary. https://www.productplan.com/glossary/weighted-shortest-job-first
- [R16] Wikipedia, "MoSCoW method". https://en.wikipedia.org/wiki/MoSCoW_method
- [R17] Medallia, "Kano model". https://www.medallia.com/experience-101/glossary/kano-model/
- [R18] Teresa Torres, Product Talk, "Opportunity Solution Trees", 2023-12-06. https://www.producttalk.org/opportunity-solution-trees/
- [R19] Atlassian, "Epics, stories, and initiatives". https://www.atlassian.com/agile/project-management/epics-stories-themes ; Atlassian, "Portfolio for JIRA FAQs". https://confluence.atlassian.com/display/JIRAPortfolioServer025/Portfolio+for+JIRA+FAQs
- [R20] bmad-code-org, BMAD-METHOD repo: skills/bmad-ticket/assets/initiative-template.md and skills/bmad/references/initiative.md (read 2026-10-09). https://github.com/bmad-code-org/BMAD-METHOD
- [R21] Ryan Singer, Basecamp, "Shape Up", chapter 8 "The Betting Table" and glossary. https://basecamp.com/shapeup/2.2-chapter-08
- [R22] Ryan Singer, Basecamp, "Shape Up", chapter 2 "Principles of Shaping". https://basecamp.com/shapeup/1.1-chapter-02
- [R23] Seth Hobson, wshobson/agents: plugins/conductor/skills/track-management/SKILL.md. https://github.com/wshobson/agents
- [R24] bmad-code-org, BMAD-METHOD README and releases (v6.12.1, 2026-10-04). https://github.com/bmad-code-org/BMAD-METHOD
- [R25] bmad-code-org, BMAD-METHOD skills/bmad-agent-pm/SKILL.md and customize.toml. https://github.com/bmad-code-org/BMAD-METHOD/tree/main/skills/bmad-agent-pm
- [R26] Mintlify mirror of the BMAD docs, "Agents" (third-party; may be stale). https://mintlify.wiki/bmad-code-org/BMAD-METHOD/concepts/agents
- [R27] BMAD docs, "Getting Started". https://docs.bmad-method.org/tutorials/getting-started ; tessl registry, bmad-check-implementation-readiness. https://tessl.io/registry/skills/github/bmad-code-org/BMAD-METHOD/bmad-check-implementation-readiness
- [R28] BMAD docs, "Choose a planning path". https://docs.bmad-method.org/plan/choose-a-planning-path/
- [R29] FoundationAgents, MetaGPT README. https://github.com/FoundationAgents/MetaGPT
- [R30] Sirui Hong et al., "MetaGPT: Meta Programming for a Multi-Agent Collaborative Framework", arXiv 2308.00352 (2023; v7 2024-11-01). https://arxiv.org/abs/2308.00352
- [R31] Chen Qian et al., "ChatDev: Communicative Agents for Software Development", arXiv 2307.07924. https://arxiv.org/html/2307.07924v5
- [R32] GitHub, Spec Kit README. https://github.com/github/spec-kit
- [R33] GitHub, Spec Kit, spec-driven.md. https://github.com/github/spec-kit/blob/main/spec-driven.md
- [R34] Kiro docs, "Specs" and "Feature specs". https://kiro.dev/docs/specs/ ; https://kiro.dev/docs/specs/feature-specs/
- [R35] Fission-AI, OpenSpec README. https://github.com/Fission-AI/OpenSpec
- [R36] VoltAgent, awesome-claude-code-subagents: categories/08-business-product/product-manager.md. https://github.com/VoltAgent/awesome-claude-code-subagents
- [R37] Factory docs, "What are Droids?". https://docs.factory.ai/user-guides/droids/understanding-droids ; eesel, "Factory AI in 2026". https://www.eesel.ai/blog/factory-ai
- [R38] Addy Osmani, "Human-in-the-Loop", Agentic Engineering (undated; (c) 2026). https://addyosmani.com/agentic-engineering/human-in-the-loop/
- [R39] FeatBit, "Human-in-the-Loop Gates for AI Agents: Approval Without Review Fatigue" (vendor blog). https://www.featbit.co/blogs/human-in-the-loop-gate-for-agents
