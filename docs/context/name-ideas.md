# Name ideas

A working list of proper names for the human's machines, and later for named agents and seats:
themes, candidates, what's been tested and what's been cut. Kept by the human, added to over time.
How to choose a name (short, survives dictation, a type is not a name) is in
[[naming]]; this file is only the pool of ideas.

Status: ideas, nothing decided. No machine has been renamed.

## Working idea (2026-10-11)

Not decided. The overall theme is space and sci-fi (the human, 2026-10-11: "our overall theme
is: space / sci-fi"). Within it, a theme per kind of machine, each with real-world and sci-fi
names, from the human's three categories ("windows and nuc don't move; their fixed in position
... and let's think of what the cloud servers would be. laptops we only have one right now,
dalek, that is slightly off-theme").

| Kind | Theme | Why |
|---|---|---|
| 1. Fixed, at home | The fixed stars | They keep their places, as home machines do. Home needs only a few names. |
| 2. Cloud | Wanderers: planets, moons, asteroids, space stations | "Planet" is Greek for wanderer: cloud servers come and go. The biggest set of names the human knows, for the kind that may grow. |
| 3. Laptops | Spaceships and spacecraft | They travel. |

The split is the ancients' own: the fixed stars stay put, and the planets wander across them. The
human swapped home and cloud from a first draft (planets at home, stars in the cloud) because home
is by far the smallest list, and they recognise more solar-system names than star names; and
"planet means wanderer".

The theme says what kind of machine it is, not where it sits, so a machine that moves keeps its
name (the c2 tip against location names, [[naming]]).

**1. Fixed, at home: the fixed stars**

- Real: `vega`, `rigel`, `altair`, `deneb`, `antares`, `procyon`, `capella`; pairs `castor` and
  `pollux`, `mizar` and `alcor`.
- Sci-fi: `canopus` (a real star; Arrakis's sun in Dune), `rao` (Krypton's red sun), `tatoo`
  (Tatooine's suns). Few, but home needs few.
- Picks: the Windows PC `rigel` or `antares` (big, bright stars for the heavy lifting); the NUC
  `vega` (short, clear, one of the brightest in the sky). The other family PC, if it joins:
  `altair` or `deneb`.
- Avoid: `polaris` and `sirius` (sound like other words), `sol` (the Sun; too short and generic).

**2. Cloud: the wanderers**

- Real planets and moons: `jupiter`, `saturn`, `mars`, `neptune`; `luna`, `europa`, `io`,
  `callisto`, `phobos`, `titan`, `rhea`, `miranda`, `charon`.
- Real asteroids: `vesta`, `pallas`, `psyche`, `bennu`.
- Real space stations: `mir`, `skylab`, `salyut`, `tiangong`.
- Sci-fi worlds: `vulcan`, `bajor`, `hoth`, `endor`, `naboo`, `caprica`, `arrakis`, `caladan`,
  `gallifrey`, `skaro`, `trantor`, `terminus`.
- Sci-fi stations: `babylon` (Babylon 5), `citadel` (Mass Effect), `yorktown` (Star Trek Beyond),
  `elysium`.
- Picks: the EC2 `luna` (the nearest one out; the EC2 is the NUC's fallback to the outside). Then
  planets and moons as servers are added.
- Avoid: `ceres` (dictation hears "series"), `earth` (generic), `uranus`, `gateway` (bridle
  already has a gateway).

**3. Laptops: ships**

- Real spacecraft: `apollo`, `voyager`, `pioneer`, `cassini`, `galileo`, `kepler`, `juno`,
  `rosetta`, `atlantis`, `soyuz`.
- Sci-fi ships: `nostromo`, `serenity`, `defiant`, `rocinante`, `leonov`, `normandy`, `moya`,
  `falcon`, `orville`, `starbug`.
- Picks: the next laptop from `nostromo`, `serenity`, `voyager`, `defiant`.
- `dalek` stays: kept by the human ("it's been dalek for a long time"), the one exception to the
  scheme.

**Galaxies: not placed yet.** A galaxy holds many stars and planets, so one idea is a galaxy name
for a group rather than a machine: the tailnet, all of bridle's machines, or a project's machines.
The list is under "Galaxies and nebulae" in Themes.

## Machines that need names

| Machine | Now | Leading idea | Notes |
|---|---|---|---|
| MacBook Pro (this one) | dalek | dalek | Keeping it: "it's been dalek for a long time" (the human). |
| Intel NUC | nuc | vega | "NUC" is a type, and dictation hears "Nook". Was `sonic`; see "Working idea". |
| Windows desktop, running WSL2 | EzekielsPC | rigel, antares | The human couldn't settle on `tardis` (2026-10-11). See "WSL2 and its name" below. |
| EC2 | (none) | luna | The edge box (the NUC's tunnel fallback). Was `terminus`. |
| Future cloud servers | | planets and moons | The wanderers. |
| Other MacBook Pros | | nostromo, serenity, voyager, defiant | Ships, because laptops travel. Only those that run bridle or join Tailscale. |
| The other family PC | | altair, deneb | Only if it joins Tailscale. |

Not needed: the family's Windows laptops, the PS5, PS4 and Switch.

Idea: one theme per kind of place (home machines one theme, cloud another), so the name says
where a machine lives without naming its hardware. Themes needn't all match; "all sci-fi opens up
a lot of really good options" (the human).

What suits a machine, the human (2026-10-11): "i like planets, stars, place names, space ships for
machine names. not generally personal names for machines; although the god names fit into several
categories and could be used." So characters and other personal names are for agent seats, not
machines.

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

**Sci-fi ships** (a favourite; the long ones sound great but run past eight letters. None
dictation-tested.)

| Name | From | Notes |
|---|---|---|
| enterprise | Star Trek | The famous one; long, and a common word. |
| voyager, defiant, reliant | Star Trek | Voyager is also the real probe. |
| excelsior, stargazer, cerritos | Star Trek | Excelsior and stargazer are long. |
| titan, protostar | Star Trek | Titan is also a moon and a god. |
| discovery | Star Trek; 2001: A Space Odyssey | Both franchises. A common word, and long. |
| orville | The Orville | |
| falcon | Star Wars (the Millennium Falcon) | Also SpaceX's rocket. |
| tantive | Star Wars (Tantive IV, Leia's ship) | |
| executor | Star Wars (Vader's flagship) | |
| ghost | Star Wars Rebels | A common word. |
| nostromo, sulaco | Alien; Aliens | |
| auriga, covenant | Alien Resurrection; Alien: Covenant | Auriga is also a constellation. |
| prometheus | Prometheus; Stargate | Also the Titan. Long. |
| serenity | Firefly | |
| rocinante | The Expanse (named after Don Quixote's horse) | Long. |
| donnager, canterbury | The Expanse | Long. |
| galactica | Battlestar Galactica | Long. |
| pegasus | Battlestar Galactica | Also a constellation. |
| leonov | 2010: The Year We Make Contact | |
| endurance | Interstellar | Long. |
| icarus | Sunshine | |
| horizon | Event Horizon | A common word. |
| endeavour | Rendezvous with Rama (Clarke) | Also the real shuttle. Spelled the British way. |
| moya | Farscape | A living ship. |
| liberator | Blake's 7 | Long. |
| starbug | Red Dwarf | |
| normandy | Mass Effect | |
| daedalus, odyssey, destiny | Stargate | Odyssey and destiny are common words. |
| highliner | Dune (the Guild's ships) | |

Real spacecraft, for the same theme:

| Name | Notes |
|---|---|
| apollo, gemini, mercury | NASA's crewed programmes; also a god, a constellation and a planet. |
| voyager, pioneer, viking | Probes that left (or landed); pioneer and viking are common words. |
| cassini, galileo, kepler, hubble, webb | Probes and telescopes named after astronomers. |
| juno, rosetta, dawn, huygens | Probes. |
| atlantis, discovery, endeavour | Shuttles. Leave out challenger and columbia: both were lost with their crews. |
| soyuz, vostok, sputnik, mir | Soviet craft and station. |

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

**Dune** (a favourite; Frank Herbert's six novels only, not the later books by Brian Herbert and
Kevin J. Anderson. Many won't work as names; listed anyway. Not dictation-tested.)

Characters:

| Name | Who | Book | Notes |
|---|---|---|---|
| paul | Paul Atreides, Muad'Dib, Usul | Dune | A common first name. |
| leto | Duke Leto Atreides; his grandson Leto II, the God Emperor | Dune; Children; God Emperor | Short. |
| jessica | Lady Jessica, Paul's mother, Bene Gesserit | Dune | |
| alia | Paul's sister, "St. Alia of the Knife" | Dune; Messiah; Children | Short; may come out "Aaliyah". |
| thufir | Thufir Hawat, the Atreides Mentat | Dune | Hard to spell. |
| gurney | Gurney Halleck, warrior-troubadour | Dune; Children | |
| duncan | Duncan Idaho, swordmaster, later ghola after ghola | all six | A common first name. |
| yueh | Dr. Wellington Yueh, the Suk doctor who betrays the Atreides | Dune | Dictation will struggle. |
| stilgar | Fremen naib of Sietch Tabr | Dune; Messiah; Children | |
| chani | Paul's Fremen partner | Dune; Messiah | May come out "Chaney". |
| kynes | Liet-Kynes, the planetologist | Dune | Sounds like "kinds". |
| jamis | the Fremen Paul kills in his first duel | Dune | |
| mapes | Shadout Mapes, the Fremen housekeeper | Dune | |
| harkonnen | Baron Vladimir Harkonnen | Dune | Also the house. |
| feyd | Feyd-Rautha, the Baron's heir | Dune | May come out "fade". |
| rabban | Glossu Rabban, "the Beast" | Dune | |
| piter | Piter de Vries, the Harkonnen Mentat | Dune | Sounds like "Peter". |
| shaddam | Shaddam IV, the Padishah Emperor | Dune | |
| irulan | Princess Irulan, whose writings open each chapter | Dune; Messiah | |
| fenring | Count Hasimir Fenring; Lady Margot Fenring | Dune | |
| mohiam | Gaius Helen Mohiam, the Reverend Mother | Dune; Messiah | |
| scytale | the Tleilaxu Face Dancer | Messiah; Chapterhouse | Hard to spell. |
| edric | the Guild Navigator | Messiah | Short and clear. |
| hayt | the first Duncan Idaho ghola | Messiah | Sounds like "hate". |
| bijaz | the Tleilaxu dwarf | Messiah | |
| korba | Fremen priest of the Qizarate | Messiah | |
| ghanima | Leto II's twin sister | Children | |
| faradn | Farad'n Corrino, heir to the Corrinos | Children | The apostrophe goes. |
| wensicia | Farad'n's mother | Children | Long. |
| preacher | the blind Preacher | Children | A common word. |
| moneo | Leto II's majordomo | God Emperor | |
| siona | Moneo's daughter, the rebel | God Emperor | |
| hwi | Hwi Noree, the Ixian ambassador | God Emperor | Dictation will struggle. |
| nayla | a Fish Speaker | God Emperor | |
| odrade | Darwi Odrade, Mother Superior | Heretics; Chapterhouse | |
| taraza | Alma Mavis Taraza, Mother Superior | Heretics | |
| teg | Miles Teg, the Bashar | Heretics | Short. |
| sheeana | the girl who commands the worms | Heretics; Chapterhouse | |
| lucilla | a Bene Gesserit Imprinter | Heretics | |
| murbella | the Honored Matre who becomes Bene Gesserit | Heretics; Chapterhouse | |
| waff | Tylwyth Waff, a Tleilaxu Master | Heretics | Sounds like "waft". |
| burzmali | a Bashar, Teg's student | Heretics | |
| bellonda | a Bene Gesserit Mentat | Chapterhouse | |
| tamalane | a senior Bene Gesserit | Chapterhouse | |
| dama | the Great Honored Matre | Chapterhouse | |

Houses (the Great Houses and the Houses Minor):

| Name | What | Notes |
|---|---|---|
| atreides | Paul's house, of Caladan | Hard to spell. |
| harkonnen | the Baron's house, of Giedi Prime | |
| corrino | the Imperial house, of Kaitain and Salusa Secundus | |
| moritani | the house of Grumman, in a feud with Ginaz | |
| ginaz | a house beaten by Grumman; later the swordmasters' school | |
| landsraad | the assembly of the houses | |

Races and powers:

| Name | What | Notes |
|---|---|---|
| fremen | the desert people of Arrakis | |
| sardaukar | the Emperor's soldier-fanatics, of Salusa Secundus | |
| guild | the Spacing Guild and its Navigators | A common word. |
| navigator | the Guild's spice-mutated pilots | A common word. |
| bene gesserit | the sisterhood: breeding programme, the Voice, Reverend Mothers | Two words. |
| tleilaxu | the Bene Tleilax: gene-masters, gholas, Face Dancers | Hard to spell and say. |
| choam | the trading company every house holds shares in | Said "choam"; dictation will guess. |
| mentat | the human computers | Clear; a fit for a build box. |
| ixians | the machine makers of Ix | `ix` alone is two letters. |
| richese | machine makers, Ix's rival | |
| suk | the Suk School of doctors | Sounds like "suck". |
| qizarate | Paul's priesthood | |
| fish speakers | Leto II's all-woman army | Two words. |
| honored matres | the women who return from the Scattering | Two words. |
| futars | the hunters bred against the Honored Matres | |
| shai-hulud | the sandworms, "Old Man of the Desert" | The hyphen goes. |

Places (not asked for, but planets are machine names elsewhere in this file):
`arrakis` (also Dune, later Rakis), `caladan`, `giedi` (Giedi Prime), `kaitain`, `salusa`, `ix`,
`tleilax`, `ecaz`, `grumman`, `gammu`, `wallach` (Wallach IX), `chapterhouse`, `junction`.

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

**Stars** (442 names in the IAU's official catalogue, so the set won't run out. Picks below;
none dictation-tested.) Lists: the IAU Catalog of Star Names
([namingschemes.com](https://namingschemes.com/IAU_Catalog_of_Star_Names_(IAU-CSN)), which blocks
automated readers, and the IAU working group's
[full text file](https://www.pas.rochester.edu/~emamajek/WGSN/IAU-CSN.txt), last updated
2022-04-04; the official page is [iau.org](https://www.iau.org/public/themes/naming_stars/)).

| Name | Notes |
|---|---|
| vega, rigel, deneb, altair, antares, procyon, canopus, regulus | Bright, well-known stars; easy to say. |
| sirius, polaris, spica, capella | Well known, but sound like other words or names: "serious", Polaris (a product), "speaker", "a cappella". |
| arcturus, aldebaran, betelgeuse | Famous, but long; betelgeuse is hard to spell. |
| castor, pollux | The twins in Gemini: a pair. |
| mizar, alcor | A double star in the Big Dipper's handle: a pair. |
| maia, electra, merope, alcyone | Sisters of the Pleiades: a family set. |
| mira, izar, thuban, rana | Short and unusual. |

**Planets, moons and asteroids** (the solar system: about 300 named moons and tens of thousands of
named asteroids, so it won't run out. Many are also gods. None dictation-tested.)

Planets and dwarf planets:

| Name | Notes |
|---|---|
| mercury, venus, mars, jupiter, saturn, neptune | Mercury and mars are also a god and a candy; jupiter is long-ish. |
| pluto, ceres, eris | Dwarf planets; short. Eris is also the goddess of strife. |
| haumea, makemake, sedna, quaoar | Far-out dwarf planets; makemake and quaoar are hard to say and spell. |
| uranus, earth | Leave out: one invites jokes, the other is too generic. |

Moons:

| Name | Moon of | Notes |
|---|---|---|
| luna | Earth | Also a common pet name. |
| phobos, deimos | Mars | A pair: fear and dread. |
| io, europa, ganymede, callisto | Jupiter (the Galilean moons) | `io` is two letters and also "I/O"; ganymede is long. |
| amalthea, himalia, metis, thebe | Jupiter | |
| titan, rhea, dione, tethys, mimas | Saturn | Titan is also a Star Trek ship. |
| enceladus, iapetus, hyperion, phoebe | Saturn | Enceladus and iapetus are hard to spell. |
| janus, pandora, atlas, prometheus | Saturn | All gods too; atlas is a common product name. |
| miranda, ariel, oberon, titania, puck, umbriel | Uranus (named from Shakespeare and Pope) | Miranda and ariel are personal names too. |
| triton, proteus, nereid, larissa | Neptune | |
| charon, nix, hydra, styx, kerberos | Pluto | Short; the underworld set. Hydra is also a constellation. |

Asteroids and other small bodies:

| Name | Notes |
|---|---|
| vesta, pallas, juno, hygiea | The four big main-belt asteroids. |
| psyche | The metal asteroid NASA's probe is visiting. |
| eros, ida, gaspra, mathilde, lutetia | Visited by probes; ida has its own moon, dactyl. |
| itokawa, ryugu, bennu | Sampled by probes, and the samples brought home. |
| didymos, dimorphos | The pair DART hit. |
| apophis | The near-Earth one; also the Egyptian serpent of chaos. |
| arrokoth | The farthest body a probe has visited (New Horizons). |

Fictional planets, from franchises already in this file:

| Name | From | Notes |
|---|---|---|
| vulcan, bajor, risa | Star Trek | |
| tatooine, hoth, endor, naboo, dagobah | Star Wars | Hoth and endor are short. |
| coruscant, alderaan | Star Wars | Long. |
| caprica | Battlestar Galactica | |
| magrathea | The Hitchhiker's Guide to the Galaxy | Long. |
| gallifrey, skaro | Doctor Who | Skaro is already a spare. |
| arrakis, caladan, trantor, terminus | Dune; Foundation | Also under those themes. |

**Galaxies and nebulae** (none dictation-tested)

Real galaxies and groups:

| Name | Notes |
|---|---|
| andromeda, triangulum | Our big neighbours; andromeda is long. |
| magellan | The Large and Small Magellanic Clouds, our satellite galaxies. |
| whirlpool, pinwheel, sombrero, cartwheel, cigar, sunflower, tadpole, antennae | Galaxies named for their shapes: everyday words, so clear to say but less distinct. |
| blackeye | The Black Eye Galaxy. |
| milkyway | Ours; run together, it's a candy bar. |
| laniakea | Our supercluster ("immeasurable heaven" in Hawaiian). Long. |
| virgo, fornax, sculptor | Clusters and dwarf galaxies; also constellations. |

Sci-fi galaxies:

| Name | From | Notes |
|---|---|---|
| pegasus | Stargate Atlantis | Also a constellation and a Galactica ship. |
| ida | Stargate SG-1 (the Asgard's home) | Also an asteroid. |
| andromeda | Mass Effect: Andromeda; the Andromeda series | |
| isop | Doctor Who ("The Web Planet") | |
| mutter | Doctor Who: Mutter's Spiral, the Time Lords' name for the Milky Way | |
| delta, gamma | Star Trek's quadrants of the Milky Way | Common words; Greek letters. |

Nebulae, the same idea in smaller clouds: `crab`, `eagle`, `horsehead`, `lagoon`, `helix`,
`ring`, `veil`, `carina`; sci-fi `mutara` (Star Trek II).

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

- **Stephen King:** liked; names in the table after this comment. Places: `derry`, `overlook`.

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

- [Jerod Santo, "Naming schemes"](https://jerodsanto.net/2012/03/naming-schemes).
- [FSFE's scheme](https://wiki.fsfe.org/Teams/System-Hackers/NamingSchemes): elements for
  hosts, their discoverers for VMs (a parent/child pattern).

- [Sci-fi hostname schemes](https://rubenerd.com/brad-alexander-on-hostname-schemes-sci-fi),
  [Fun with Host Naming Schemes](https://www.steevithak.com/?p=808).
- [RFC 1178](https://www.rfc-editor.org/rfc/rfc1178), "Choosing a Name for Your Computer".

- Generators: [The Story Shack](https://thestoryshack.com/tools/server-hostname-generator/)
  (servers, ships, Foundation characters), fantasynamegenerators.com.
- A lesson from sysadmins: one favourite novel runs out of names fast; pick a category with
  room (starships, planets).
