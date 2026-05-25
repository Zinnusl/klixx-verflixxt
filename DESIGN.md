# Klixx Designnotizen

## Prämisse

`Klixx` ist ein Fan-Point-and-Click-Adventure über Verflixxte Klixx. Die
spielende Person rät nicht nur Videoklicks, sondern untersucht `Folge 000`:
eine verschollene Folge ohne verlässlichen Zeitpunkt. Ein Host ist nach dem
Öffnen eines Archivframes nicht zurückgekommen; die Regie behandelt Studio,
Stadt und Videoarchiv als zusammenhängenden Fall.

Die Struktur bleibt ein Mystery-Adventure mit klassischem Verb-Objekt-Spiel.
Die Inspiration aus Zeitreise-Adventures liegt in der Fallarbeit: Gegenwart
markieren, Signale benennen, eine harte Uhrzeit sichern, Licht als Stadtanker
zurückbringen und den instabilen Videoframe erst betreten, wenn die Hinweise im
Raum verstanden sind. Es werden keine Figuren, Dialoge oder Szenen aus anderen
Spielen übernommen.

## Technik

- Rust-Spielcode in `src/main.rs`.
- Macroquad/miniquad für die Canvas-Runtime.
- Trunk für Dev-Server und Web-Builds.
- Projektlokales JavaScript-Plugin für Browser-`localStorage`.
- `web/index.html` lädt `klixx.wasm` über den Macroquad-Loader.
- `assets/fonts/DejaVuSans.ttf` wird als Default-Font gesetzt, damit Umlaute
  sicher sichtbar sind.

## Oberfläche

Das Spiel nutzt eine klassische Adventure-Hülle:

- Obere Leiste: Bereich, Szenenname, Statuszeile und Neustart.
- Bühne: 16:9-Raumansicht mit beschrifteten Platzhalter-Sprites.
- Footer: Verben, Inventar und Fallakte.
- Dialogfenster: Sprecher, Einstiegstext, Antwortfeld und Auswahlmöglichkeiten.
- Abschlussfenster: Der Rettungslauf für Fall 000 wird freigegeben.
- Spielfigur: einfache Click-to-walk-Bewegung innerhalb jedes Raums.

## RBTV-Studio

Die Studio-Szene ist jetzt als eigene Fan-Nachbildung gezeichnet:

- Wand, Boden, Lichttraverse und Regiefenster
- Klixx-Tisch als zentraler Spielort
- Chat-/Host-Vorschau als unsicherer Rückholpunkt
- Studiokamera, Greenscreen und Rückhol-Setup
- Bodenmarken, die den Host wieder in die Gegenwart zwingen sollen

Das ist bewusst keine offizielle Asset-Kopie, sondern eine stilisierte
Adventure-Kulisse.

## Bürogebäude

Die Heinrichstraßen-Umgebung ist präziser angelegt, soweit öffentliche Quellen
das hergeben:

- Hof mit Haus 9, Haus 11 und Haus 15 als getrennte Gebäudeteile
- Büroflur mit Studiotür, Redaktion, Küche, Besprechung und Serverraum
- Dispo-Tafel und Techniklager kurz hinter dem Eingang
- Treppenhaus zum Redaktions-/Schlauchbüro
- Serverraum 11 mit Racks, Video-Hub, Backup-Stream und 80-m-SDI-Spur zum
  Studio 9
- Haus-15-Szenenbau als ehemalige Werkstatt mit Kulissen, Maltisch und
  Zusatzstudio

Es gibt keine öffentliche, belastbare Raum-für-Raum-Grundrissquelle im Projekt.
Darum bleiben Maße und genaue Türpositionen absichtlich als Fan-Interpretation
markiert, während öffentlich belegte Elemente und Laufwege bevorzugt werden.

## Ton

Die Texte sollen nach Verflixxte Klixx klingen, ohne echte Dialoge oder Clips
zu kopieren:

- Zeitmarken werden wie Produktionsnotizen behandelt: trocken, prüfbar und
  trotzdem unheimlich.
- Der Chat hat weiterhin zu viel Meinung, aber die Falllogik entsteht aus
  Hotspots, Inventargegenständen und Dialogen.
- Metadaten, Thumbnails, Licht und Signalwege sind Beweise, nicht Lore-Text.
- Riskante Verben haben Konsequenzen: Strom, Sprinkler, Requisitensturz oder
  Archiv-Rettungsstation.
- Studiomöbel, Kabel und Büroflure kommentieren die Handlung, ohne echte
  Dialoge oder Clips zu kopieren.

Der Humor entsteht aus ernst gespielter Regelpanik: Produktionsraum-Quatsch,
Klixx-Rituale, Zeitanker und Adventure-Logik treffen aufeinander, während jede
falsche Interaktion so wirkt, als müsse sie später im Protokoll erklärt werden.

## Cameos

Cameos bleiben bewusst als originale, nicht kopierte Archivmotive:

- Heinrichstraße-Hof, Falltafel, Techniklager, Serverraum 11, 80-m-SDI-Rolle
  und Haus-15-Szenenbau
- Schanzenstraße, Sternschanze, Wasserturm und Karoviertel als Stadtanker
- Schimmelbrüder-Frame als begehbarer Videoarchivraum
- Kofferreflektor, Kofferschild und Copyshop als kleines Nebenrätsel

## Aktueller Spielabschnitt

1. Fallakte 000 an der Falltafel nehmen.
2. Gaffer-Tape im Szenenbau holen und Bodenmarken am Rückhol-Setup setzen.
3. SDI-Label im Serverraum nehmen und die Signalroute beschriften.
4. Host-Karte im Copyshop holen und am Grafikplatz laden.
5. Schimmelbrüder-Frame in der Regie betreten: Hallenboden prüfen,
   Frame-Probe nehmen, an Formenreihe und Musterbahn abgleichen, dann am
   Clip-Ausgang benutzen.
6. Bahnhofsuhr als Zeitmarke und Wasserturmlicht als Lichtmarke dokumentieren.
7. Drehhinweis lesen, Pappstück und Transparentband holen, Kofferschild drucken,
   Entlassungsbogen als Nachweis beim Straßenmusiker zeigen und den
   Kofferreflektor erhalten.
8. Reflektor am Rückhol-Setup platzieren und den Rettungslauf starten.

## Konsequenzrouten

Falsche gefährliche Verben führen nicht nur zu Gags, sondern zu kurzen
Rückkehrpuzzles:

- offene Technik: Krankenhauszimmer mit Monitor, Rufknopf, Pflegekraft und
  Entlassungsbogen
- Produktionsmaterial mit Sprinklerfolge: Evakuierungsbereich mit Alarmfeld,
  Löschschrank und Sicherheitsdienst
- instabile Requisiten: blockierter Szenenbau mit Stützstrebe und Notleine
- Archivsignal: Rettungsplatz mit Laufwerk, Prüfsummenzettel und Terminal

## Nächste Designarbeit

- Platzhalterblöcke durch eigene Fan-Art ersetzen.
- Host-Echos und Produktionsstimmen weiter als nicht-wörtliche Figuren
  ausarbeiten.
- Weitere öffentlich belegbare Details des RBTV-Gebäudes sammeln, bevor Räume
  näher an echte Layouts gerückt werden.
