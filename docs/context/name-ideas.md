# Name ideas

A working list of proper names for the human's machines, and later for named agents and seats:
themes, candidates, what's been tested and what's been cut. Kept by the human, added to over time.
How to choose a name (short, survives dictation, a type is not a name) is in
[[naming]]; this file is only the pool of ideas.

Status: ideas, nothing decided. No machine has been renamed.

## Machines that need names

| Machine | Now | Leading idea | Notes |
|---|---|---|---|
| MacBook Pro (this one) | dalek | dalek | Keeping it: "it's been dalek for a long time" (the human). |
| Intel NUC | nuc | sonic | "NUC" is a type, and dictation hears "Nook". |
| Windows desktop, running WSL2 | (none) | tardis | Bigger on the inside: Linux inside Windows. See "WSL2 and its name" below. |
| EC2 | (none) | terminus | Foundation's edge planet; the EC2 is the edge box (the NUC's tunnel fallback). |
| Future cloud servers | | trantor, gaia, aurora, solaria | More Foundation worlds. |
| Other MacBook Pros | | serenity, nostromo | Ships, because laptops travel. Only those that run bridle or join Tailscale. |
| The other family PC | | skaro | Only if it joins Tailscale. |

Not needed: the family's Windows laptops, the PS5, PS4 and Switch.

Idea: one theme per kind of place (home machines one theme, cloud another), so the name says
where a machine lives without naming its hardware. Themes needn't all match; "all sci-fi opens up
a lot of really good options" (the human).

## Advisor's suggestions (2026-10-09)

Not decided; the human decides when the WSL2 PC is set up.

**Machines: a theme per kind of place.**

- **Home, always on: Doctor Who.** `dalek` (the MacBook Pro, kept), `sonic` (the NUC), `tardis` (the PC).
  All three passed both dictation tests. Spares: `skaro`, then the companions.
- **Cloud, far away: Foundation worlds.** `terminus` for the EC2, then `trantor`, `gaia`,
  `aurora`, `solaria`. Asimov has plenty more. Not yet dictation-tested: say `terminus` and
  `trantor` in both systems before choosing.
- **Laptops, which travel: sci-fi ships.** `serenity`, `nostromo`. Only when one needs a name; `dalek` stays the exception.
- **Why not the others for machines:** constellations are hard to type (`cygnus`,
  `cassiopeia`); zodiac animals are everyday words ("send it to dog"); the gods are the
  fallback cloud theme if Foundation wears thin (`janus` or `bifrost` would suit the EC2).

**Agent seats: characters, not places.** Character names suit agents better than machines, and
the human already marked Root for seats. Following the seats ticket
([[seats-every-role-is-a-named-tracked-seat-that-outlives-its-s-gtzx|gtzx]], P3):

- **Unique roles keep the role as their name** (`orchestrator`, `aide`). A generic name says what
  the seat does ([[naming]]: generic names for what a thing is). A display name is optional.
- **Advisors with a focus are named by topic** (`advisor/naming`, `advisor/research`): clearer
  than a character.
- **General-purpose extra seats take Root's Vagabonds:** `ranger`, `tinker`, `ronin`, `arbiter`,
  `harrier`, `thief`. They roam and help whoever needs it, which is what a spare advisor does.
- **If seats get display names, use Root factions:** e.g., `marquise` for the orchestrator (she
  runs the board), `corvid` for the aide (a watchful messenger).
- **Workers and managers stay as IDs.** They're many and short-lived, and names would run out.
- **Keep for later:** Alien, Firefly and Stephen King characters, for long-lived seats that
  outgrow Root (`ripley`, `roland`, `kaylee`).

## WSL2 and its name

By default a WSL2 distro takes the Windows computer's name as its hostname, but on the network it
is a separate machine: a small VM behind NAT with its own address. (Mirrored networking shares the
Windows address instead.)

- **Set it by hand** in `/etc/wsl.conf` (`[network]`, `hostname = tardis`); restart WSL with
  `wsl --shutdown` to apply.
- **Tailscale is where it counts.** The WSL2 ticket puts Tailscale inside the Linux distro,
  because bridle reads `tailscale ip -4` there
  ([[run-bridle-s-heavy-work-on-the-windows-pc-under-wsl2-v7ug|v7ug]]). That makes WSL its own
  Tailscale node, named after its hostname, and `tardis` is the name bridle and dalek will use.
- **If Windows also runs Tailscale** (e.g., for Remote Desktop), it's a second node. Two nodes
  with one name make Tailscale rename one (`tardis-1`), so give the Windows side its own name or
  keep Tailscale off it.
- **Simplest:** name the Windows computer `tardis` too, and run Tailscale only inside WSL. One
  name everywhere.

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

> [!comment] c7 human, 2026-10-08 21:43 EDT, on "Themes" [read 2026-10-08 21:51 EDT]
> Also add the Chinese zodiac as a theme. I already asked for Greek, Norse, and Roman gods, so I was going to say the constellations, but it might be hard to type. Just go and add another one for constellations. This is a lot of fun.
>
> **doc-name-ideas-e11e82, 2026-10-08 22:00 EDT:** @human Added two themes under "In: liked": Chinese zodiac and Constellations (most are easy to type; the long ones are marked). I found no Greek, Norse or Roman gods in this file, so that earlier request may have gone to someone else. Want me to add them here too? [read 2026-10-09 05:41 EDT]

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

> [!comment] c6 human, 2026-10-08 21:40 EDT, on "Sci-fi" [read 2026-10-08 21:51 EDT]
> Add another section here with characters from the Alien franchise and Firefly.
>
> **doc-name-ideas-e11e82, 2026-10-08 22:00 EDT:** @human Added "Sci-fi characters" right after the ships table, with Alien and Firefly names, from memory. [read 2026-10-08 22:04 EDT]

| Name | From | Notes |
|---|---|---|
| nostromo | Alien | |
| serenity | Firefly | |
| galactica | Battlestar Galactica | Long. |
| rocinante | The Expanse (named after Don Quixote's horse) | Not seen yet. Long to type. |
| discovery | 2001: A Space Odyssey | Also a common word. |

**Sci-fi characters** (Alien and Firefly; first names, easy to say)

| Name | From | Notes |
|---|---|---|
| ripley, newt, hicks, bishop, ash, kane, dallas, lambert, parker | Alien | Ash and bishop are everyday words. |
| ellen, vasquez, hudson, burke, jones | Alien | `jones` is the cat. |
| mal, zoe, wash, jayne, kaylee, inara | Firefly | |
| simon, river, book, niska, badger | Firefly | `river` is also under Doctor Who. |

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

**Chinese zodiac** (twelve animals; short and dictation-safe, but common words)

| Name | Notes |
|---|---|
| rat, ox, tiger, rabbit, dragon, snake | |
| horse, goat, monkey, rooster, dog, pig | Goat is sometimes sheep or ram. |

**Constellations** (room for many more; the odd spellings are the risk)

| Name | Notes |
|---|---|
| orion, lyra, draco, vela, carina, hydra | Short; hydra is also a software name. |
| perseus, pegasus, phoenix, centaurus, aquila | Easy to say. |
| andromeda, cassiopeia | Long to type. |
| cygnus | Dictation may hear "signus". |

**Gods: Greek, Norse and Roman** (big pantheons, so plenty of room; many are also product,
planet or Marvel names)

| Pantheon | Name | Notes |
|---|---|---|
| Greek | zeus, hera, athena, apollo, artemis, hermes | Hermes is the messenger: suits mail. Apollo and atlas are common product names. |
| Greek | poseidon, hades, demeter, hestia, nike, eros | Hestia is the hearth: a home server. Nike is a brand. |
| Greek | atlas, prometheus, hyperion, gaia | Titans. Gaia is also under Foundation. |
| Greek | ares | Sounds like "Aries" (the zodiac sign and constellation). |
| Greek | hephaestus, dionysus | Hard to spell and type. |
| Norse | odin, freya, frigg, baldur, mimir, sif | Mimir is the well of wisdom: a docs or search box. |
| Norse | heimdall, bifrost | The watchman and the rainbow bridge: suit a gateway or tunnel. |
| Norse | asgard, valhalla | Places. |
| Norse | thor, loki | Marvel's too; thor may come out as "four". |
| Norse | tyr, njord, yggdrasil | Dictation and typing risks ("tyr" sounds like "tier"). |
| Roman | jupiter, juno, minerva, diana, vesta, janus | Janus has two faces, looking both ways: suits a gateway. Vesta is the hearth. |
| Roman | mercury, mars, venus, neptune, pluto, saturn, ceres | Also planets, and some are everyday words. |
| Roman | vulcan, bacchus, aurora | Vulcan is Star Trek's too. Aurora is also under Foundation. |

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
- **Silo:** watched; names to add. From the books and show: `juliette` (the engineer), `solo`
  (a common word), `bernard`, `lukas`, `walker`, `shirley`, `knox`, `jahns` (the mayor), `silo`
  itself (a type, not a name). Most are plain first names, so dictation should be easy.

> [!comment] c4 human, 2026-10-08 19:52 EDT, on "names to add" [read 2026-10-08 20:07 EDT]
> look up some names and add them here.
>
> **doc-name-ideas-e11e82, 2026-10-08 20:15 EDT:** @human Added Silo names to its bullet, from what I know of the books and show; I could not check them against a site. I read "names to add" as the Silo line (the comment sits under it). The Stephen King line says the same; want me to fill that one too? [read 2026-10-08 21:37 EDT]
>
> **resolved by human, 2026-10-08 21:37 EDT**

- **Stephen King:** liked; names in the table after this comment. Places: `derry`, `overlook`.

> [!comment] c5 human, 2026-10-08 21:38 EDT, on "Stephen King" [read 2026-10-08 21:51 EDT]
> Fill out this section with major characters from:
> - The Shining
> - The Stand
> - Carrie
> - Cujo
> - Thinner
> - Shawshank Redemption 
> - all of the four seasons
> - The Gunslinger series
>
> **doc-name-ideas-e11e82, 2026-10-08 22:00 EDT:** @human Filled in a "Stephen King" table just below, one group per work, from memory. Names shared between books (tommy, billy, chris) are listed once, under the first. I cut the bullet down to point at it. [read 2026-10-08 22:04 EDT]
>
> **resolved by human, 2026-10-09 05:41 EDT**

| Work | Names |
|---|---|
| The Shining | jack, wendy, danny, hallorann, grady, tony |
| The Stand | flagg, abagail, stu, frannie, larry, nick, harold, glen, nadine, lloyd, kojak |
| Carrie | carrie, margaret, sue, tommy, billy |
| Cujo | cujo, donna, tad, vic |
| Thinner | halleck, ginelli, lemke, heidi |
| Shawshank (Rita Hayworth and Shawshank Redemption) | andy, red, norton, hadley, brooks |
| Apt Pupil | todd, dussander |
| The Body | gordie, teddy, vern |
| The Breathing Method | mccarron |
| The Gunslinger (Dark Tower) | roland, eddie, susannah, jake, oy, cuthbert, alain, susan, blaine, mordred, sheemie |

Dictation risks: `red`, `jack`, `oy` (sounds like "oi"), `flagg` ("flag"), `kojak`. `roland` and `flagg` are the strongest.

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

- [namingschemes.com](https://namingschemes.com): a wiki of naming schemes (Starships, Superheroes, Elements, ...).

> [!comment] c3 human, 2026-10-08 19:34 EDT, on "namingschemes.com" [read 2026-10-08 20:07 EDT]
> Make this an HTML link.
>
> **doc-name-ideas-e11e82, 2026-10-08 20:15 EDT:** @human Done. It is a Markdown link, which Obsidian and the UI render as an HTML link. If you meant a literal `<a href>` tag, say so. [read 2026-10-08 21:37 EDT]
>
> **resolved by human, 2026-10-08 21:37 EDT**

- [Jerod Santo, "Naming schemes"](https://jerodsanto.net/2012/03/naming-schemes).
- [FSFE's scheme](https://wiki.fsfe.org/Teams/System-Hackers/NamingSchemes): elements for
  hosts, their discoverers for VMs (a parent/child pattern).

> [!comment] c1 human, 2026-10-08 19:33 EDT, on "elements for hosts, their discoverers for VMs" [read 2026-10-08 20:07 EDT]
> The site is a static copy. Can you find the same information on the new site and update the link here?
>
> > This page has been moved to docs.fsfe.org with the rest of the sysadmin documentation
>
> **doc-name-ideas-e11e82, 2026-10-08 20:15 EDT:** @human I could not find it. docs.fsfe.org/en/sysadmin/ answers 403 to every path I tried (real or not, so it looks login-only), and the public part (techdocs, teams/system-hackers) has no naming page. I left the link as is. If you can open it in a browser, send me the URL and I will swap it in. [read 2026-10-08 21:37 EDT]
>
> **resolved by human, 2026-10-08 21:37 EDT**

- [Sci-fi hostname schemes](https://rubenerd.com/brad-alexander-on-hostname-schemes-sci-fi),
  [Fun with Host Naming Schemes](https://www.steevithak.com/?p=808).
- [RFC 1178](https://www.rfc-editor.org/rfc/rfc1178), "Choosing a Name for Your Computer".

> [!comment] c2 human, 2026-10-08 19:34 EDT, on "RFC 1178" [read 2026-10-08 20:07 EDT]
> Please link the RFC.
>
> **doc-name-ideas-e11e82, 2026-10-08 20:15 EDT:** @human Done: linked to rfc-editor.org (checked, it loads).
>
> **resolved by doc-name-ideas-e11e82, 2026-10-08 20:15 EDT**

- Generators: [The Story Shack](https://thestoryshack.com/tools/server-hostname-generator/)
  (servers, ships, Foundation characters), fantasynamegenerators.com.
- A lesson from sysadmins: one favourite novel runs out of names fast; pick a category with
  room (starships, planets).
