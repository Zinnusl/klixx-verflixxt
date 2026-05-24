# Klixx Designnotizen

## Prämisse

`Klixx` ist ein Fan-Point-and-Click-Adventure über Verflixxte Klixx. Die
spielende Person rät nicht nur Videoklicks, sondern untersucht `Folge 000`:
eine verschollene Folge ohne Upload-Datum, ohne Laufzeit, ohne Thumbnail und mit
einem Klickzähler, der zu reagieren scheint, bevor jemand einen Tipp abgibt.

Die Struktur bleibt ein Mystery-Adventure, aber die Beweisführung klingt jetzt
wie eine Klixx-Runde, die sich selbst ernst nimmt: Ein normales Studio trifft
auf eine Klickzahl, die schon vor der Frage getippt hat. Jeder Gegenstand ist
ein Vorwand für übermutige Schätzungen, nachträgliche Begründungen,
Fischkarten-Reue oder Chat-Rechthaberei.

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
- Footer: Verben, Inventar und Klixx-Akte.
- Dialogfenster: Sprecher, Einstiegstext, Antwortfeld und Auswahlmöglichkeiten.
- Abschlussfenster: Der erste Klixx-Mystery-Beat öffnet sich.
- Spielfigur: einfache Click-to-walk-Bewegung innerhalb jedes Raums.

## RBTV-Studio

Die Studio-Szene ist jetzt als eigene Fan-Nachbildung gezeichnet:

- Wand, Boden, Lichttraverse und Regiefenster
- Klixx-Tisch als zentraler Spielort
- Bohnensofa als Cameo-Fläche
- Chat-Monolith mit Live-Kommentaren
- zwei Kameras und Kabelsalat
- Bohnenbecher, Fischkartenstapel, Preis-Sockel und Regel-Einspieler

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

- Zahlen werden erst gefühlt, dann behauptet und nach der Auflösung verteidigt.
- Die Fischkarte ist keine Mechanik, sondern eine schlechte Idee mit Ritualrang.
- Der Chat hat immer recht, oft gleichzeitig in beide Richtungen und fast immer
  mit einer Null zu viel.
- Direkttreffer wirken wie übernatürliche Ereignisse, überbotene Tipps wie
  persönliche Niederlagen.
- Metadaten, Thumbnails und Seitenverhältnisse sind Beweise, aber alle tun so,
  als sei das Bauchgefühl.
- Studiomöbel, Kabel und Büroflure kommentieren nicht die Handlung, sondern die
  Schätzpsychologie dahinter.

Der Humor entsteht aus ernst gespielter Regelpanik: Produktionsraum-Quatsch,
Klixx-Rituale und Adventure-Logik treffen aufeinander, während jede Zahl so
wirkt, als müsse sie gleich im Chat verteidigt werden.

## Cameos

Aktuelle Cameos sind bewusst als originale Archivobjekte umgesetzt:

- Fischkarte und Risikomodus
- Geierkönig-Krone
- Cheater-Tafel der Community
- Telefonzettel aus einem alten Clip
- leerer Sockel für den fantastischen Preis
- Regel-Einspieler mit Schlafmasken
- Spezialgebiet-Mappe und Format-Analyse
- Folgenspuren wie `Folge 66` und `Folge 74`
- Bohnensofa, Chat-Monolith, Kamerakran und Kabelsalat
- Heinrichstraße-Hof, Dispo-Tafel, Techniklager, Serverraum 11, 80-m-SDI-Rolle
  und Haus-15-Szenenbau

## Erster Spielabschnitt

1. `Folge 000` an der Produktionsklappe untersuchen.
2. Fischkarte vom Studiotisch nehmen.
3. Fischkarte am Regie-Klickzähler benutzen.
4. Formatnotiz aus der Format-Analyse nehmen.
5. Fehlenden Playlist-Eintrag am Archivterminal untersuchen.
6. Formatnotiz am Undercover-Frame benutzen.
7. Geborgenen Undercover-Frame am Anomalie-Monitor benutzen.

Der Abschluss öffnet `Die Folge tippt zurück`: Der Monitor gibt keinen Hinweis,
sondern einen viel zu selbstsicheren ersten Tipp. Danach folgt man der
unmöglichen Klickzahl aus dem Studio heraus.

## Zweiter Spielabschnitt

Nach dem ersten Abschluss erscheint in der Regie die `Signalspur`. Sie führt in
das `Klickrauschen`: keinen echten Raum, sondern eine begehbare Interpretation
des Videofehlers.

1. Signalspur hinter dem Anomalie-Monitor betreten.
2. Leeren Schätzzettel nehmen.
3. Mindestens zwei Spuren untersuchen: Ritual-Meter, Archivwelle,
   Chat-Rauschen.
4. Schätzzettel am Klick-Orakel benutzen.
5. Die erste Schätzung stabilisieren: `23 Klicks`, ohne eine Angstnull vom Chat
   mitzunehmen.

Das Schätzrätsel soll sich wie Klixx anfühlen, aber Adventure-Logik bleiben:
Die Lösung entsteht aus Spuren im Raum, nicht aus externem Wissen über echte
Klickzahlen. Falschoptionen sind keine Mathefehler, sondern typische
Klixx-Impulsfehler: zu viel Archivdramaturgie oder eine Null aus Panik.

## Nächste Designarbeit

- Entscheiden, ob die Spielfigur ein originales Crewmitglied, ein Zuschauer oder
  eine abstrakte Cursor-Präsenz im Archiv ist.
- Platzhalterblöcke durch eigene Fan-Art ersetzen.
- Host-Echos und Produktionsstimmen weiter als nicht-wörtliche Figuren
  ausarbeiten.
- Weitere öffentlich belegbare Details des RBTV-Gebäudes sammeln, bevor Räume
  näher an echte Layouts gerückt werden.
