# Klixx

Ein Fan-Point-and-Click-Adventure inspiriert von `Verflixxte Klixx`, gebaut mit
Rust/Macroquad und ausgeliefert über Trunk.

## Starten

```bash
./scripts/dev.sh
```

Dann `http://127.0.0.1:8080/` öffnen.

Release-Build:

```bash
./scripts/build_web.sh
./scripts/serve_web.sh
```

## Aktuelle Richtung

Klixx ist ein deutschsprachiges Point-and-Click-Abenteuer um `Fall 000`: Ein
Host verschwindet nach dem Öffnen eines Archivframes in einer begehbaren
Videospur. Die zunächst nüchterne Studioarbeit wird zu einem Rettungslauf mit
Zeitanker-Logik: Heinrichstraße, Serverraum, Haus-15-Szenenbau, Schanzenstraße,
Bahnhof Sternschanze, Schanzenpark, Karoviertel und der Schimmelbrüder-Frame
werden Teil desselben Falls.

Aktueller Stand:

- Rust-Spielcode in `src/main.rs`.
- Macroquad/miniquad als Browser-Canvas-Runtime.
- Trunk-Dev-Server und Web-Build-Pipeline.
- Projektlokale Browser-Speicherung per `localStorage`.
- Klassische Adventure-Oberfläche mit Statusleiste, Bühne, Kontextaktion,
  Inventar, Fallakte, Dialogfenster und Click-to-walk-Bewegung.
- Eigene Unicode-Schrift für sichere Darstellung von Umlauten.
- Neu generierte Pixel-Look-Hintergründe für zehn Szenen, inklusive
  Greenscreen-Studio, Regie, Produktionshof, Serverraum, Szenenbau und vier
  Stadtorten.
- Animierte Praktikantenfigur als 4x4-Walkcycle-Spritesheet.
- Generierte Inventaricons für Fallakte, Gaffer-Tape, SDI-Label, Host-Karte,
  Lichtreflektor und die Archiv-/Nebenquest-Gegenstände.
- Spielziel: Fallakte holen, Bodenmarken setzen, Signalweg beschriften,
  Host-Karte in der Regie laden, den Schimmelbrüder-Frame stabilisieren,
  Bahnhofstakt und Parklicht prüfen, den Kofferreflektor beschaffen und den
  Rettungslauf starten.

## Steuerung

- Verb wählen, dann Hotspot anklicken: Ansehen, Anstubsen, Benutzen,
  Aufheben, Anzüngeln, Riechen oder Reden.
- Ausgänge funktionieren unabhängig vom aktuell gewählten Verb.
- Auf freie Bodenfläche klicken, um die Spielfigur zu bewegen.
- Inventargegenstand anklicken, dann kompatiblen Hotspot anklicken.
- `Neu` löscht den lokalen Spielstand.

## Hotspot-Editor

```bash
./scripts/dev.sh
```

Dann `http://127.0.0.1:8080/hotspot-editor.html` öffnen. Der Editor lädt die
Szenenbilder, zeigt die aktuellen Klickflächen, erlaubt ein Walkable-Polygon
für die Spielfigur und exportiert angepasste `pct(...)`-/`walkable`-Werte für
`src/main.rs`.

Der Button `Definitionen speichern` schreibt die aktuellen Editorwerte direkt
in `src/main.rs` und in die Editor-Defaults. Dafür startet `./scripts/dev.sh`
lokal zusätzlich `http://127.0.0.1:8082/`.

Über `trunk serve` ist derselbe Editor auch unter
`http://127.0.0.1:8080/hotspot-editor/` erreichbar.

## Frame-Picker

```bash
./scripts/dev.sh
```

Dann `http://127.0.0.1:8080/frame-picker/` öffnen. Der Picker lädt die lokalen
Videos aus `tmp/klixx-video-rooms/videos/`, zeigt sie in einem Video-Scrubber
mit Timeline und extrahiert am aktuellen Timecode per Button einen Frame. Die
Frame-Liste darunter zeigt standardmäßig nur selbst gepickte Frames und scrollt
unabhängig vom Video-Scrubber; automatisch extrahierte Vorschläge bleiben über
den Listenfilter optional erreichbar. Frames können als `Use`, `Reject` und
`Walkable` markiert werden. Die Auswahl wird nach
`tmp/klixx-video-rooms/frame_selections.json` sowie
`assets/selected_video_frames.json` gespeichert.

Lokale Forschungskopien der Katalogvideos können bei Bedarf unter `tmp/` landen:

```bash
./scripts/klixx_video_rooms.py download --limit 0
```

Der Default lädt bewusst eine moderate Qualität. Für einen Testlauf ohne
Download:

```bash
./scripts/klixx_video_rooms.py download --limit 0 --dry-run
```
