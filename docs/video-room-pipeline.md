# Video Room Pipeline

The story direction is: one host is lost inside the Klixx videos, and the
player enters stylized versions of those videos as puzzle locations.

The workflow is intentionally research-first:

1. collect candidate video URLs and episode notes,
2. extract representative still frames locally,
3. review contact sheets and choose the best room candidates,
4. use selected frames only as composition/reference material,
5. generate original, playable point-and-click rooms with clear walkable space
   and embedded hotspots.

Do not ship source videos or raw screenshots as game assets. Final rooms should
be new, stylized interpretations rather than copied frames.

## Setup

`ffmpeg` and `ffprobe` are required. `yt-dlp` is optional globally; this project
can fetch a local copy into `tmp/`:

```bash
./scripts/fetch_yt_dlp.sh
```

## Discover Candidates

```bash
./scripts/klixx_video_rooms.py discover --search --per-query 1
```

This writes:

```text
tmp/klixx-video-rooms/catalog.json
```

Current discovery sources:

- curated seed URLs for known Klixx episodes and compilations,
- RBTV forum playlist collection,
- fernsehserien.de episode-guide pages used as motif notes,
- YouTube search queries for specific Klixx episodes/motifs.

Search results are filtered so broad Rocket-Beans hits without a Klixx title do
not enter the catalog.

## Extract Frames

```bash
./scripts/klixx_video_rooms.py extract --limit 5 --frames-per-video 4
./scripts/klixx_video_rooms.py contact-sheet
```

This writes:

```text
tmp/klixx-video-rooms/frames/
tmp/klixx-video-rooms/frames.json
tmp/klixx-video-rooms/contact_sheet.jpg
```

The extraction streams video URLs through `ffmpeg` and stores only still frames.
By default a new extraction clears the previous frame folder so the contact
sheet reflects the current run. Use `--append` only when intentionally building
one larger sheet across several runs.

Useful focused scans:

```bash
./scripts/klixx_video_rooms.py extract --offset 10 --limit 10 --frames-per-video 3
./scripts/klixx_video_rooms.py extract --video-id 2NBCX4yW5tk --frames-per-video 8
./scripts/klixx_video_rooms.py extract --video-id 2NBCX4yW5tk --seconds 1089 --append
```

For the frame-picker workflow, `--limit 0` means all catalog videos:

```bash
./scripts/klixx_video_rooms.py download --limit 0
./scripts/klixx_video_rooms.py extract --limit 0 --frames-per-video 12
```

The browser picker is available through the dev server. It has two layers:
first a video-editor style scrubber for the locally downloaded videos, then a
scrolling review grid. The grid defaults to manually picked frames; previously
extracted candidate frames are still available through the list filter.

```text
http://127.0.0.1:8080/frame-picker/
```

Selections are saved to:

```text
tmp/klixx-video-rooms/frame_selections.json
assets/selected_video_frames.json
```

The first curated room brief pass lives in:

```text
assets/video_room_briefs.json
```

It maps selected video frames to planned generated room assets and normalized
click targets.

## Implemented Slice

`video_kliemannsland_road` is the first playable video room:

- generated/local fallback asset: `assets/scenes/video_kliemannsland_road.png`,
- editor entry: `web/hotspot-editor.html`,
- route: Regie -> Straßenvideo,
- puzzle: inspect the walkable road plane, take the route marker, inspect the
  red-white road markings for depth order, use the route marker on the archive
  exit sign.

`video_schimmelbrueder` is the first manually picked frame promoted into a
playable video room:

- selected reference frame: `tmp/klixx-video-rooms/frames/-cf1yCs-m8Q_verflixxte_hits_die_besten_songs_aus_200_folgen_verflixxte_klixx/pick_000075975ms.jpg`,
- generated/local fallback asset: `assets/scenes/video_schimmelbrueder.png`,
- editor entry: `web/hotspot-editor.html`,
- route: Regie -> Schimmelbrüder-Video,
- puzzle: inspect the factory floor, take the form sample, read the patterned
  belt, use the sample on the refrain gate.

The AI image-generation tool failed with a generic `UserError` during this pass,
so the first slice uses a local pixel-art fallback asset. The room is wired so a
future generated replacement can overwrite the same scene asset path.

`video_sewer_archive` remains documented as a deprecated experiment only. The
source frame looks into a narrow pipe and was rejected because it does not offer
human-scale standing space.

## Selection Criteria

Good room candidates have:

- a visible standing/walking plane in the source frame,
- a strong location or object silhouette,
- enough visual structure to become a walkable room,
- no dependency on real faces or licensed logos,
- a clear Klixx rule hook: Fischkarte, Undercovervideo, Telefonjoker,
  Deutsch-Bonus, Geierkönig, Direkttreffer, Greenscreen, Timecode, or Joker.

## Prompt Pattern

For a selected reference frame:

```text
Use this frame only as a composition and location reference.
Create an original 1990s cinematic pixel-art point-and-click room inspired by
the location type, lighting and object layout. Do not copy the screenshot, do
not include real people, readable logos, watermarks or exact copyrighted UI.
Make it a playable room: lower 40% walkable, embedded hotspots, clear exits,
The Dig-like serious mood, subtle Klixx-inspired puzzle objects.
```

Current walkable-frame shortlist:

```text
assets/walkable_video_frames.json
tmp/klixx-video-rooms/walkable_candidate_sheet.jpg
```

The sewer-pipe frame is explicitly rejected for future room generation because
the original frame does not contain human-scale standing space.
