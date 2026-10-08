# Name ideas

A working list of proper names for the human's machines, and later for named agents and seats:
themes, candidates, what's been tested and what's been cut. Kept by the human, added to over time.
How to choose a name (short, survives dictation, a type is not a name) is in
[[naming]]; this file is only the pool of ideas.

Status: ideas, nothing decided. No machine has been renamed.

## Machines that need names

| Machine | Now | Leading idea | Notes |
|---|---|---|---|
| Mac (this one) | dalek | dalek | Keep, or change later. |
| Intel NUC | nuc | sonic | "NUC" is a type, and dictation hears "Nook". |
| Windows desktop, running WSL2 | (none) | tardis | Bigger on the inside: Linux inside Windows. Use one name for Windows and WSL. |
| EC2 | (none) | ? | Needs a name now. |
| Future cloud servers | | ? | Pick a theme with room for more. |
| Other MacBook Pros | | ? | Only those that run bridle or join Tailscale. |
| The other family PC | | ? | Only if it joins Tailscale. |

Not needed: the family's Windows laptops, the PS5, PS4 and Switch.

Idea: one theme per kind of place (home machines one theme, cloud another), so the name says
where a machine lives without naming its hardware. Themes needn't all match; "all sci-fi opens up
a lot of really good options" (the human).

## Dictation tests

2026-10-07, first with Claude's dictation, then with Wispr Flow. Every name passed in both, in
sentences and after "at" (as in `aide@tardis`):

| Name | Claude | Wispr Flow |
|---|---|---|
| tardis | TARDIS | Tardis |
| sonic | Sonic | Sonic |
| dalek | Dalek | Dalek |

Seen along the way: "aide" alone came out as "aid" in both ("send a message to aid", "aid@Sonic"),
but right as "the aide at Sonic".

## Themes

### In: liked

**Doctor Who** (the home theme so far)

| Name | From | Notes |
|---|---|---|
| dalek | the Daleks | Taken (the Mac). |
| sonic | sonic screwdriver | Small, handy tool. |
| tardis | the Doctor's ship | Bigger on the inside. |
| clara, amy, rose, donna, martha | companions | Ordinary first names, so dictation is easy. |
| skaro | the Daleks' planet | Spare. |
| river | companion | A common word, so less clear. |
| k9 | the robot dog | A digit; dictation may say "canine". |

**Sci-fi ships** (a favourite; some are long to type, but they sound great)

| Name | From | Notes |
|---|---|---|
| nostromo | Alien | |
| serenity | Firefly | |
| galactica | Battlestar Galactica | Long. |
| rocinante | The Expanse (named after Don Quixote's horse) | Not seen yet. Long to type. |
| discovery | 2001: A Space Odyssey | Also a common word. |

**Foundation** (read the whole series; "all making my cut for now")

| Name | What | Notes |
|---|---|---|
| trantor | the capital planet | |
| terminus | the Foundation's planet | Suits an edge or relay box, like the EC2. |
| seldon | Hari Seldon | |
| gaia | the living planet | |
| daneel | R. Daneel Olivaw, the robot | |
| mule | the Mule | A common word. |
| aurora, solaria | Spacer worlds | |

**Arthur C. Clarke** (reading now)

| Name | From |
|---|---|
| hal | 2001 |
| discovery | 2001 (also in the ships) |
| rama | Rendezvous with Rama |

**Root** (the board game, which the family plays a lot, with every expansion): for named agents and
seats rather than machines. Faction and character names, to check against the boxes:

| Name | Faction or piece | Animal |
|---|---|---|
| marquise | Marquise de Cat | cats |
| eyrie | Eyrie Dynasties | birds (dictation may hear "Erie" or "aerie") |
| alliance | Woodland Alliance | mice, bunnies, foxes |
| vagabond | Vagabond | raccoon and others |
| riverfolk | Riverfolk Company | otters |
| cult | Lizard Cult | lizards |
| duchy | Underground Duchy | moles |
| corvid | Corvid Conspiracy | crows |
| hundreds | Lord of the Hundreds | rats |
| keepers | Keepers in Iron | badgers |
| lilypad | Lilypad Diaspora | frogs |
| twilight | Twilight Council | bats |

Vagabond characters: thief, tinker, ranger, arbiter, scoundrel, vagrant, adventurer, ronin,
harrier. Short and dictation-safe, but many are everyday words.

### Maybe

- **Hitchhiker's Guide:** liked, but only `marvin` would get used, and it's a little obscure.
- **Silo:** watched; names to add.
- **Stephen King:** liked; names to add (e.g., `derry`, `roland`, `gunslinger`).

### Hard on dictation: liked, but likely to fail

- **The Three-Body Problem** (read all of it; ties in with China): e.g., `sophon`, which will
  likely come out as "Sofon" or "sofa".
- **Three Kingdoms** characters: the human once named everything after them.
- **Chinese places and myth** (`wukong`, `xian`, `chengdu`).

### Not yet read

Red Rising (soon), Brandon Sanderson (next; less into fantasy).

### Out

- **Marvel AIs** (jarvis, edith, vision, friday): "Cut that off." Marvel heroes otherwise:
  `thor` and `groot` are risky for dictation.
- **Shakespeare** (hamlet, othello, ophelia): used a lot before (they were IU's mail server
  names), but "it got old".

## Places to find more

- namingschemes.com: a wiki of naming schemes (Starships, Superheroes, Elements, ...).

> [!comment] c3 human, 2026-10-08 19:34 EDT, on "namingschemes.com" [pending 2026-10-08 19:34 EDT]
> Make this an HTML link.

- [Jerod Santo, "Naming schemes"](https://jerodsanto.net/2012/03/naming-schemes).
- [FSFE's scheme](https://wiki.fsfe.org/Teams/System-Hackers/NamingSchemes): elements for
  hosts, their discoverers for VMs (a parent/child pattern).

> [!comment] c1 human, 2026-10-08 19:33 EDT, on "elements for hosts, their discoverers for VMs" [pending 2026-10-08 19:33 EDT]
> The site is a static copy. Can you find the same information on the new site and update the link here?
>
> > This page has been moved to docs.fsfe.org with the rest of the sysadmin documentation

- [Sci-fi hostname schemes](https://rubenerd.com/brad-alexander-on-hostname-schemes-sci-fi),
  [Fun with Host Naming Schemes](https://www.steevithak.com/?p=808).
- RFC 1178, "Choosing a Name for Your Computer".

> [!comment] c2 human, 2026-10-08 19:34 EDT, on "RFC 1178" [pending 2026-10-08 19:34 EDT]
> Please link the RFC.

- Generators: [The Story Shack](https://thestoryshack.com/tools/server-hostname-generator/)
  (servers, ships, Foundation characters), fantasynamegenerators.com.
- A lesson from sysadmins: one favourite novel runs out of names fast; pick a category with
  room (starships, planets).
