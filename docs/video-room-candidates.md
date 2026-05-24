# Video Room Candidates

Story premise: one Klixx host is lost inside the video archive. The player uses
the studio as a gateway, enters individual videos as spatial puzzle rooms, and
rebuilds the route by solving Klixx-rule problems inside those rooms.

The first implementation pass uses these candidates as video sources for frame
extraction. Raw frames stay under `tmp/`; final game rooms must be original
pixel-art interpretations with no copied UI, faces, logos or readable source
text.

## Sources

- Bohnenwiki format/rules/VOD context: https://bohnen.wiki/index.php/Verflixxte_Klixx
- RBTV forum playlist collection: https://forum.rocketbeans.tv/t/playlist-sammlung-aus-verflixxte-klixx/4974
- fernsehserien episode-guide motif checks: https://www.fernsehserien.de/verflixxte-klixx/episodenguide/staffel-2/42372/8

## Room Seeds

| Candidate | URL | Room hook |
|---|---|---|
| Kliemannsland #30 | https://www.youtube.com/watch?v=yuQxtOc6UjU | Outdoor hub, gate, yard, map route. |
| Kanalisation #184 | https://youtu.be/2NBCX4yW5tk | Pipe tunnel, grates, flow direction, hidden clue. |
| Murmiland #188 | https://youtu.be/Z6iXrBWRtYo | Marble-track logic, switches, path timing. |
| Piñata #180 | https://youtu.be/SqI0Ub-ZwyQ | Blindfold, stick, hanging object, wrong-hit puzzle. |
| Merry Klixx-Mas #190 | https://youtu.be/2KHducit5Ko | Gift/decor rule room with seasonal edge cases. |
| Merry KlixxMas #178 | https://youtu.be/dtYwV0y1UdY | Warm holiday room, drink/decor/inventory puzzle. |
| Merry KlixxMas #151 | https://www.youtube.com/watch?v=zxRw5goiOzI | Music/decor variation for a second holiday layer. |
| Scheiben/Paviane #191 | https://youtu.be/suxqQx_CK5E | Glass panels, reflection, nested animal-video clue. |
| Delphine #96 | https://www.youtube.com/watch?v=y3Gs9bXlvcw | Water/sonar/signal puzzle. |
| Nilpferd #95 | https://www.youtube.com/watch?v=LNPaIeTqwX0 | Large central obstruction and weight logic. |
| Wassermelonen #98 | https://www.youtube.com/watch?v=vBntrQG_-jw | Falling object, weight, breakage and timing. |
| Moms & Militärs #94 | https://www.youtube.com/watch?v=Vbc3Va7xUNU | Map room, equipment labels, instruction order. |
| Waffel #97 | https://www.youtube.com/watch?v=IcxWmBDG51I | Kitchen/snack object puzzle. |
| Gnocchis #149 | https://www.youtube.com/watch?v=glgp4J8_Mlw | Fryer, timer, ingredients and heat controls. |
| Müll #142 | https://www.youtube.com/watch?v=Hev5fxAMQ5c | Trash/recycling search room. |
| Angeln #133 | https://www.youtube.com/watch?v=jw5ogHDggOk | Hook, bait, line tension, retrieval puzzle. |
| Gefährlichster Anruf #158 | https://youtu.be/s71eKnq9hs8 | Telephone room with dial and cable logic. |
| Unbekannte Nummern #174 | https://youtu.be/9QlFlAAbgwA | Mobile-phone contact and number puzzle. |
| Daten-Diebstahl #187 | https://youtu.be/Kn6AdumKdMY | Phishing/password/user-interface puzzle. |
| Harry Potter #167 | https://youtu.be/_1dfIOwmYqk | Book, letter and rule-magic room. |
| Macht #105 | https://www.youtube.com/watch?v=vqy7t6cYIeo | Sci-fi light, helmet, starfield motif. |
| Geierkönig #169 | https://youtu.be/0PCL1nXHlIE | Crown, throne, trophy and meta-score puzzle. |
| Jäger der verlorenen Klixx #110 | https://www.youtube.com/watch?v=30aT2C6N1zA | Artifact, map and adventure-temple room. |
| Keller/Beutel #189 | https://youtu.be/JoxfZbQhgwM | Dark storage room, bag inventory clue. |
| Klixx #1 | https://www.youtube.com/watch?v=hh47NNnS_Ik | Origin room for the base rule system. |
| Jubiläum #150 | https://www.youtube.com/watch?v=Kk9OCIV_ygk | Trophy/meta-room with archive objects. |
| 4:3 #163 | https://youtu.be/PvfTc_TInII | CRT, aspect ratio, archive-video puzzle. |
| Feedback/Vogel #192 | https://youtu.be/EpEKYwPEkMw | Undercover-video and feedback-room logic. |
| Verflixxte Hits | https://www.youtube.com/watch?v=-cf1yCs-m8Q | Song room, broken audio, rule jingle. |
| 10 Jahre Live | https://www.youtube.com/watch?v=At4AiO7YTeo | Finale/meta room around the archive gateway. |
| Fischkarten-Intros | https://www.youtube.com/watch?v=hb9R7GDFbJ4 | Fischkarte altar, wager, score multiplier. |
| Anrufe-Best-of | https://www.youtube.com/watch?v=s3ucU3chDrE | Telephone-joker exchange and wrong connection. |

## Writing Hooks

- Fischkarte: risk/reward object that doubles progress but can strand the route.
- Deutschmultiplikator: courtroom-like rule dispute about whether a video counts.
- Undercovervideo: fake clip hidden in a playlist, detectable through production errors.
- Missing episode: a room made of gaps, blur fields and cut marks.
- Bildformat: 4:3, vertical video and stretched UI reveal different hotspot layers.
- Telefonjoker: a switchboard links clips, callers and wrong rooms.
- Geierkönig: a meta-score throne room that tracks victories but blocks the rescue if
  pride is chosen over cooperation.
- Regel-Einspieler: a projected rules sequence changes the room state after playback.
