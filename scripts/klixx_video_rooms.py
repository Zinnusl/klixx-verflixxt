#!/usr/bin/env python3
"""Research pipeline for turning Klixx-related videos into room references.

The script stores metadata, optional local video downloads, still frames and
contact sheets under tmp/. It does not place source videos in the repo.
"""

from __future__ import annotations

import argparse
import html
import json
import math
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse

import requests
from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "tmp" / "klixx-video-rooms"
CATALOG = OUT / "catalog.json"
FRAMES_JSON = OUT / "frames.json"
VIDEOS_DIR = OUT / "videos"
DOWNLOADS_JSON = OUT / "downloads.json"
VIDEO_EXTENSIONS = (".mp4", ".webm", ".mkv", ".mov")

FORUM_PLAYLIST_TOPIC = "https://forum.rocketbeans.tv/t/playlist-sammlung-aus-verflixxte-klixx/4974.json"
EPISODE_GUIDES = [
    "https://www.fernsehserien.de/verflixxte-klixx/episodenguide/staffel-2/42372/7",
    "https://www.fernsehserien.de/verflixxte-klixx/episodenguide/staffel-2/42372/8",
    "https://www.fernsehserien.de/verflixxte-klixx/episodenguide/staffel-2/42372/9",
]

SEARCH_QUERIES = [
    "Verflixxte Klixx Folge 162 NDR Team",
    "Verflixxte Klixx Folge 172 Deutschmultiplikator",
    "Verflixxte Klixx Folge 176 Schnupperpraktikant Manfred",
    "Verflixxte Klixx Folge 182 Saxofonist Regie Skandal",
    "Verflixxte Klixx Folge 190 Fischkartenfluch",
    "Verflixxte Klixx Folge 195 Greenscreen Helden",
    "Verflixxte Klixx Folge 200 Kanalisation",
    "Verflixxte Klixx Folge 201 Blitz Elektrizität",
    "Verflixxte Klixx Folge 202 vergessenes Undercovervideo",
    "Verflixxte Klixx Folge 210 Fischkarte",
    "Verflixxte Klixx 10 Jahre Klixx Live Joker",
    "Best of Rocket Beans alle Fischkarten Intros Verflixxte Klixx",
    "Best of Rocket Beans alle Anrufe Verflixxte Klixx",
]


@dataclass
class VideoCandidate:
    kind: str
    title: str
    url: str
    source: str
    note: str = ""
    episode_hint: str = ""

    def as_dict(self) -> dict[str, str]:
        return {
            "kind": self.kind,
            "title": self.title,
            "url": self.url,
            "source": self.source,
            "note": self.note,
            "episode_hint": self.episode_hint,
        }


def seed(title: str, url: str, hint: str, note: str) -> VideoCandidate:
    return VideoCandidate(
        kind="seed-video",
        title=title,
        url=url,
        source="curated subagent/web research seed",
        note=note,
        episode_hint=hint,
    )


SEED_VIDEOS = [
    seed(
        "Verflixxte Klixx Staffel 2 aus dem Kliemannsland #30",
        "https://www.youtube.com/watch?v=yuQxtOc6UjU",
        "Kliemannsland / Scheunentor",
        "Outdoor hub candidate with a strong real-world location and walkable yard shape.",
    ),
    seed(
        "Das Video ist so deep wie die Kanalisation & schmutzig #184",
        "https://youtu.be/2NBCX4yW5tk",
        "Kanalisation",
        "Sewer and mud motif for a pipe, grate and hidden-object puzzle room.",
    ),
    seed(
        "AUF GEHT'S ins Murmiland! #188",
        "https://youtu.be/Z6iXrBWRtYo",
        "Murmiland",
        "Marble-track/mechanical model candidate with visible path logic.",
    ),
    seed(
        "Piñata Challenge: Da kann SO VIEL SCHIEF GEHEN! #180",
        "https://youtu.be/SqI0Ub-ZwyQ",
        "Piñata",
        "Object-centric challenge with blindfold, stick and candy-like affordances.",
    ),
    seed(
        "Wunderlich willkürliche Weihnachtsvideos! / Merry Klixx-Mas #190",
        "https://youtu.be/2KHducit5Ko",
        "Merry Klixx-Mas",
        "Holiday prop room candidate with gifts, decorations and rule edge cases.",
    ),
    seed(
        "Merry KlixxMas! Mehr Weihnachten geht nicht! #178",
        "https://youtu.be/dtYwV0y1UdY",
        "Merry KlixxMas",
        "Warm seasonal room candidate with drinks, ornaments and celebration props.",
    ),
    seed(
        "Merry KlixxMas Teil 2: Jetzt wird frohlockt! #151",
        "https://www.youtube.com/watch?v=zxRw5goiOzI",
        "Merry KlixxMas Teil 2",
        "Alternate holiday chapter source for music/decor interactions.",
    ),
    seed(
        "Schäbige Scheiben + popelnde PAVIAN-POPOS! #191",
        "https://youtu.be/suxqQx_CK5E",
        "Scheiben / Paviane",
        "Glass, panels and animal-video motifs for surreal nested-video rooms.",
    ),
    seed(
        "Was Delphine uns lehren... #96",
        "https://www.youtube.com/watch?v=y3Gs9bXlvcw",
        "Delphine",
        "Water/sonar candidate that can become an aquarium or signal puzzle.",
    ),
    seed(
        "Das Nilpferd im Raum #95",
        "https://www.youtube.com/watch?v=LNPaIeTqwX0",
        "Nilpferd",
        "Large central-object candidate for a room-scale obstruction puzzle.",
    ),
    seed(
        "Wie sind die Wassermelonen so gefallen? #98",
        "https://www.youtube.com/watch?v=vBntrQG_-jw",
        "Wassermelonen",
        "Physics candidate with falling, weight and breakage as puzzle language.",
    ),
    seed(
        "Verflixxte Moms & Militärs #94",
        "https://www.youtube.com/watch?v=Vbc3Va7xUNU",
        "Militär",
        "Map, equipment and instruction-room candidate.",
    ),
    seed(
        "Achtung, ich habe eine Waffel! #97",
        "https://www.youtube.com/watch?v=IcxWmBDG51I",
        "Waffel",
        "Food/object gag candidate that can become a kitchen-counter puzzle.",
    ),
    seed(
        "Frittierte Gnocchis #149",
        "https://www.youtube.com/watch?v=glgp4J8_Mlw",
        "Gnocchis",
        "Kitchen/fryer candidate with timer, ingredients and heat controls.",
    ),
    seed(
        "Was ist mit dem f****** Müll? #142",
        "https://www.youtube.com/watch?v=Hev5fxAMQ5c",
        "Müll",
        "Trash/recycling search-room candidate with layered containers.",
    ),
    seed(
        "Wer angelt sich den Sieg? #133",
        "https://www.youtube.com/watch?v=jw5ogHDggOk",
        "Angeln",
        "Fishing-line, bait and pull-mechanic candidate.",
    ),
    seed(
        "Der gefährlichste Anruf der Klixx-Geschichte #158",
        "https://youtu.be/s71eKnq9hs8",
        "Telefon",
        "Phone-room candidate for dial, cable and contact-list puzzles.",
    ),
    seed(
        "2 Männer, 2 unbekannte Nummern & ein Handy #174",
        "https://youtu.be/9QlFlAAbgwA",
        "Handy / unbekannte Nummern",
        "Mobile-phone/contact logic candidate.",
    ),
    seed(
        "DREISTER DATEN-DIEBSTAHL per Telefon?! #187",
        "https://youtu.be/Kn6AdumKdMY",
        "Telefon / Daten",
        "Phishing/data-theft puzzle source with UI and password affordances.",
    ),
    seed(
        "Harry Potter in unserer Show #167",
        "https://youtu.be/_1dfIOwmYqk",
        "Magie",
        "Book, letter and wand-like prop candidate for a magic-rule room.",
    ),
    seed(
        "Mit wem wird die Macht sein? #105",
        "https://www.youtube.com/watch?v=vqy7t6cYIeo",
        "Sci-Fi",
        "Sci-fi light, helmet or starfield motif candidate.",
    ),
    seed(
        "Die Rückkehr des Geierkönigs! #169",
        "https://youtu.be/0PCL1nXHlIE",
        "Geierkönig",
        "Crown/throne/trophy candidate for the Klixx meta-progression.",
    ),
    seed(
        "Jäger der verlorenen Klixx #110",
        "https://www.youtube.com/watch?v=30aT2C6N1zA",
        "Abenteuer",
        "Artifact, temple and map candidate for an adventure-chapter room.",
    ),
    seed(
        "Mund abwischen, Punkte im Beutel, Keller zuschei*en #189",
        "https://youtu.be/JoxfZbQhgwM",
        "Keller / Beutel",
        "Dark basement/storage candidate with bag and hidden-note puzzles.",
    ),
    seed(
        "Verflixxte Klixx #1 mit Lars Paulsen und Florentin Will",
        "https://www.youtube.com/watch?v=hh47NNnS_Ik",
        "Ursprungsfolge",
        "Origin room for the simplest version of the Klixx rule system.",
    ),
    seed(
        "150 Folgen Verflixxte Klixx - die große Jubiläumssendung",
        "https://www.youtube.com/watch?v=Kk9OCIV_ygk",
        "Jubiläum",
        "Meta-room candidate with trophies, retrospection and rule objects.",
    ),
    seed(
        "Das ist 4:3 - Karriere ist vorbei! #163",
        "https://youtu.be/PvfTc_TInII",
        "4:3 / Bildformat",
        "CRT, aspect-ratio and archive-video puzzle candidate.",
    ),
    seed(
        "SO gibt man freundliches Feedback, DU VOGEL! #192",
        "https://youtu.be/EpEKYwPEkMw",
        "Bananenkatze / Vogel",
        "Surreal object candidate for an undercover-video and feedback room.",
    ),
    seed(
        "Verflixxte Hits | Die besten Songs aus 200 Folgen Verflixxte Klixx",
        "https://www.youtube.com/watch?v=-cf1yCs-m8Q",
        "Songs compilation",
        "Compilation candidate with varied visual inserts.",
    ),
    seed(
        "10 Jahre VERFLIXXTE KLIXX: Die Jubiläumsfolge LIVE",
        "https://www.youtube.com/watch?v=At4AiO7YTeo",
        "10 Jahre live",
        "Anniversary candidate for a finale/meta framing room.",
    ),
    seed(
        "Best of Rocket Beans | Alle Fischkarten-Intros aus 10 Jahren Verflixxte Klixx",
        "https://www.youtube.com/watch?v=hb9R7GDFbJ4",
        "Fischkarten",
        "Fischkarte motif source for an inventory/puzzle chapter.",
    ),
    seed(
        "Best of Rocket Beans | Alle Anrufe aus 10 Jahren Verflixxte Klixx",
        "https://www.youtube.com/watch?v=s3ucU3chDrE",
        "Anrufe",
        "Phone-call motif source for a telephone-joker chapter.",
    ),
]


def run(cmd: list[str], *, timeout: int | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def ytdlp_bin() -> str:
    configured = os.environ.get("YTDLP_BIN")
    if configured:
        return configured
    local = ROOT / "tmp" / "yt-dlp"
    if local.exists():
        return str(local)
    found = shutil.which("yt-dlp")
    if found:
        return found
    raise SystemExit("yt-dlp not found. Run ./scripts/fetch_yt_dlp.sh first.")


def video_id(url: str) -> str:
    parsed = urlparse(url)
    if parsed.hostname and "youtu.be" in parsed.hostname:
        return parsed.path.strip("/")
    query_id = parse_qs(parsed.query).get("v", [""])[0]
    return query_id or re.sub(r"[^a-zA-Z0-9_-]+", "_", url)[-24:]


def slug(text: str) -> str:
    cleaned = re.sub(r"[^a-zA-Z0-9]+", "_", text.lower()).strip("_")
    return cleaned[:70] or "video"


def title_is_klixx_related(title: str, query: str = "") -> bool:
    haystack = f"{title} {query}".lower()
    if "game two" in title.lower() and "klixx" not in title.lower():
        return False
    return any(
        marker in haystack
        for marker in [
            "klixx",
            "verflixxte",
            "fischkarte",
            "fischkarten",
            "anrufe",
            "lars und florentin",
            "lars & florentin",
        ]
    )


def dedupe_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    deduped: list[dict[str, Any]] = []
    seen: set[str] = set()
    for entry in entries:
        url = entry.get("webpage_url") or entry.get("url") or ""
        key = video_id(url)
        if not key or key in seen:
            continue
        seen.add(key)
        deduped.append(entry)
    return deduped


def fetch_forum_playlist() -> list[VideoCandidate]:
    response = requests.get(FORUM_PLAYLIST_TOPIC, timeout=20)
    response.raise_for_status()
    data = response.json()
    found: list[VideoCandidate] = []
    seen: set[str] = set()
    for post in data.get("post_stream", {}).get("posts", []):
        cooked = post.get("cooked", "")
        for raw_url in re.findall(r"https://www\.youtube\.com/watch\?[^\"'\s<>]+", cooked):
            url = html.unescape(raw_url)
            vid = video_id(url)
            if vid in seen:
                continue
            seen.add(vid)
            found.append(
                VideoCandidate(
                    kind="source-video",
                    title=f"Forum playlist video {vid}",
                    url=f"https://www.youtube.com/watch?v={vid}",
                    source="RBTV forum playlist collection",
                    note="Direct candidate from the community playlist collection.",
                )
            )
    return found


def fetch_episode_notes() -> list[dict[str, str]]:
    notes: list[dict[str, str]] = []
    for url in EPISODE_GUIDES:
        response = requests.get(url, timeout=20)
        response.raise_for_status()
        text = re.sub(r"\s+", " ", response.text)
        for term in [
            "NDR",
            "Deutsch",
            "Praktikant",
            "Fischkarte",
            "Greenscreen",
            "Kanalisation",
            "Blitz",
            "Undercover",
            "Joker",
        ]:
            if term.lower() in text.lower():
                notes.append({"source": url, "term": term})
    return notes


def search_youtube(queries: list[str], per_query: int) -> list[VideoCandidate]:
    yt = ytdlp_bin()
    found: list[VideoCandidate] = []
    seen: set[str] = set()
    for query in queries:
        target = f"ytsearch{per_query}:{query}"
        try:
            proc = run([yt, "--dump-json", "--no-playlist", target], timeout=90)
        except subprocess.CalledProcessError as exc:
            print(f"search failed: {query}: {exc.stderr.strip()}", file=sys.stderr)
            continue
        for line in proc.stdout.splitlines():
            if not line.strip():
                continue
            info = json.loads(line)
            url = info.get("webpage_url") or info.get("original_url")
            title = info.get("title") or query
            if not url:
                continue
            if not title_is_klixx_related(title, query):
                print(f"skip unrelated search result: {title}", file=sys.stderr)
                continue
            vid = video_id(url)
            if vid in seen:
                continue
            seen.add(vid)
            found.append(
                VideoCandidate(
                    kind="episode-or-compilation",
                    title=title,
                    url=url,
                    source=f"ytsearch: {query}",
                    note="Search result for episode-level frame extraction.",
                    episode_hint=query,
                )
            )
    return found


def enrich_with_metadata(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    yt = ytdlp_bin()
    enriched: list[dict[str, Any]] = []
    for entry in entries:
        url = entry.get("url")
        if not url:
            enriched.append(entry)
            continue
        try:
            proc = run([yt, "--dump-json", "--no-playlist", url], timeout=90)
            info = json.loads(proc.stdout.splitlines()[-1])
            merged = dict(entry)
            merged.update(
                {
                    "title": info.get("title") or entry.get("title"),
                    "duration": info.get("duration"),
                    "channel": info.get("channel") or info.get("uploader"),
                    "thumbnail": info.get("thumbnail"),
                    "webpage_url": info.get("webpage_url") or url,
                }
            )
            enriched.append(merged)
        except Exception as exc:  # keep catalog usable even when YouTube blocks one item
            merged = dict(entry)
            merged["metadata_error"] = str(exc)
            enriched.append(merged)
    return enriched


def discover(args: argparse.Namespace) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    entries = [item.as_dict() for item in SEED_VIDEOS]
    entries.extend(item.as_dict() for item in fetch_forum_playlist())
    if args.search:
        entries.extend(item.as_dict() for item in search_youtube(SEARCH_QUERIES, args.per_query))
    entries = dedupe_entries(entries)
    entries = enrich_with_metadata(entries)
    episode_notes = fetch_episode_notes()
    payload = {
        "generated_by": "scripts/klixx_video_rooms.py discover",
        "notes": [
            "Videos are used as research references only.",
            "Generated rooms should be original, stylized adventure spaces rather than screenshots.",
        ],
        "episode_note_sources": episode_notes,
        "videos": entries,
    }
    CATALOG.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"wrote {CATALOG} with {len(entries)} video candidates")


def media_url(page_url: str) -> str:
    yt = ytdlp_bin()
    proc = run(
        [
            yt,
            "-g",
            "-f",
            "best[height<=720]/best",
            "--no-playlist",
            page_url,
        ],
        timeout=90,
    )
    lines = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    if not lines:
        raise RuntimeError(f"no media URL returned for {page_url}")
    return lines[-1]


def local_video_path(vid: str) -> Path | None:
    if not VIDEOS_DIR.exists():
        return None
    matches = [
        path
        for path in VIDEOS_DIR.iterdir()
        if path.is_file() and path.suffix.lower() in VIDEO_EXTENSIONS and path.name.startswith(f"{vid}_")
    ]
    return sorted(matches)[0] if matches else None


def frame_times(duration: int | float | None, count: int) -> list[float]:
    if not duration or duration < 20:
        return [4.0, 8.0, 12.0][:count]
    safe_duration = float(duration)
    if safe_duration > 1800:
        # For full episodes, skip intro/setup and sample across the middle.
        start, end = 0.18, 0.80
    else:
        start, end = 0.15, 0.90
    if count <= 1:
        fractions = [(start + end) * 0.5]
    else:
        step = (end - start) / (count - 1)
        fractions = [start + step * index for index in range(count)]
    return [max(2.0, min(safe_duration - 2.0, safe_duration * f)) for f in fractions]


def selected_catalog_videos(args: argparse.Namespace) -> list[dict[str, Any]]:
    if not CATALOG.exists():
        raise SystemExit(f"{CATALOG} missing. Run discover first.")
    payload = json.loads(CATALOG.read_text(encoding="utf-8"))
    videos = payload.get("videos", [])
    if args.video_id:
        wanted = set(args.video_id)
        videos = [
            entry
            for entry in videos
            if video_id(entry.get("webpage_url") or entry.get("url") or "") in wanted
        ]
    else:
        limit = None if args.limit == 0 else args.limit
        videos = videos[args.offset : None if limit is None else args.offset + limit]
    return videos


def download(args: argparse.Namespace) -> None:
    videos = selected_catalog_videos(args)
    if not videos:
        raise SystemExit("no videos selected")

    VIDEOS_DIR.mkdir(parents=True, exist_ok=True)
    archive = VIDEOS_DIR / "downloaded.txt"
    yt = ytdlp_bin()
    downloads: list[dict[str, Any]] = []

    for index, entry in enumerate(videos, 1):
        url = entry.get("webpage_url") or entry.get("url")
        if not url:
            continue
        vid = video_id(url)
        output_template = str(VIDEOS_DIR / "%(id)s_%(title).80s.%(ext)s")
        cmd = [
            yt,
            "--no-playlist",
            "--restrict-filenames",
            "--download-archive",
            str(archive),
            "--write-info-json",
            "--no-progress",
            "--print",
            "after_move:filepath",
            "-f",
            args.format,
            "-o",
            output_template,
            url,
        ]
        if args.max_filesize:
            cmd[1:1] = ["--max-filesize", args.max_filesize]
        if args.dry_run:
            cmd[1:1] = ["--simulate"]

        print(f"[{index}/{len(videos)}] {vid} {entry.get('title')}")
        print(" ".join(cmd))
        if args.dry_run:
            continue

        try:
            proc = run(cmd, timeout=args.timeout)
            files = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
            downloads.append(
                {
                    "video_id": vid,
                    "title": entry.get("title"),
                    "url": url,
                    "files": [str(Path(file).relative_to(ROOT)) if Path(file).is_absolute() and Path(file).is_relative_to(ROOT) else file for file in files],
                }
            )
        except subprocess.CalledProcessError as exc:
            print(f"download failed: {url}: {exc.stderr.strip()}", file=sys.stderr)

    if not args.dry_run:
        DOWNLOADS_JSON.write_text(
            json.dumps(downloads, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        print(f"wrote {DOWNLOADS_JSON} with {len(downloads)} download entries")


def extract(args: argparse.Namespace) -> None:
    if not CATALOG.exists():
        raise SystemExit(f"{CATALOG} missing. Run discover first.")
    payload = json.loads(CATALOG.read_text(encoding="utf-8"))
    videos = payload.get("videos", [])
    frames: list[dict[str, Any]] = []
    frame_root = OUT / "frames"
    if frame_root.exists() and not args.append:
        shutil.rmtree(frame_root)
    frame_root.mkdir(parents=True, exist_ok=True)

    selected_videos = selected_catalog_videos(args)
    for entry in selected_videos:
        url = entry.get("webpage_url") or entry.get("url")
        if not url:
            continue
        vid = video_id(url)
        title_slug = slug(entry.get("title") or vid)
        video_dir = frame_root / f"{vid}_{title_slug}"
        video_dir.mkdir(parents=True, exist_ok=True)
        local_video = None if args.stream else local_video_path(vid)
        if local_video:
            direct = str(local_video)
        else:
            try:
                direct = media_url(url)
            except Exception as exc:
                print(f"media url failed: {url}: {exc}", file=sys.stderr)
                continue
        times = args.seconds or frame_times(entry.get("duration"), args.frames_per_video)
        for index, seconds in enumerate(times, 1):
            out = video_dir / f"{index:02d}_{int(seconds):05d}s.jpg"
            cmd = [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                f"{seconds:.2f}",
                "-i",
                direct,
                "-frames:v",
                "1",
                "-vf",
                f"scale={args.width}:-1",
                "-y",
                str(out),
            ]
            try:
                run(cmd, timeout=60)
                frames.append(
                    {
                        "video_id": vid,
                        "title": entry.get("title"),
                        "url": url,
                        "source": entry.get("source"),
                        "episode_hint": entry.get("episode_hint"),
                        "seconds": seconds,
                        "path": str(out.relative_to(ROOT)),
                    }
                )
                print(f"frame {out}")
            except Exception as exc:
                print(f"frame failed: {url} @ {seconds:.1f}s: {exc}", file=sys.stderr)
    if args.append and FRAMES_JSON.exists():
        previous = json.loads(FRAMES_JSON.read_text(encoding="utf-8"))
        frames = previous + frames
    FRAMES_JSON.write_text(json.dumps(frames, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"wrote {FRAMES_JSON} with {len(frames)} frames")


def contact_sheet(args: argparse.Namespace) -> None:
    if FRAMES_JSON.exists():
        entries = json.loads(FRAMES_JSON.read_text(encoding="utf-8"))
        frame_files = [ROOT / entry["path"] for entry in entries if (ROOT / entry["path"]).exists()]
    else:
        frame_files = sorted((OUT / "frames").glob("*/*.jpg"))
    if not frame_files:
        raise SystemExit("no frames found. Run extract first.")
    thumbs: list[tuple[Path, Image.Image]] = []
    for path in frame_files:
        img = Image.open(path).convert("RGB")
        img.thumbnail((args.thumb_width, args.thumb_height))
        canvas = Image.new("RGB", (args.thumb_width, args.thumb_height), (12, 12, 12))
        x = (args.thumb_width - img.width) // 2
        y = (args.thumb_height - img.height) // 2
        canvas.paste(img, (x, y))
        thumbs.append((path, canvas))

    cols = args.columns
    label_h = 34
    rows = math.ceil(len(thumbs) / cols)
    sheet = Image.new(
        "RGB",
        (cols * args.thumb_width, rows * (args.thumb_height + label_h)),
        (18, 18, 18),
    )
    draw = ImageDraw.Draw(sheet)
    for idx, (path, img) in enumerate(thumbs):
        col = idx % cols
        row = idx // cols
        x = col * args.thumb_width
        y = row * (args.thumb_height + label_h)
        sheet.paste(img, (x, y))
        draw.rectangle([x, y + args.thumb_height, x + args.thumb_width, y + args.thumb_height + label_h], fill=(28, 24, 22))
        label = path.parent.name[:32]
        draw.text((x + 6, y + args.thumb_height + 5), f"{idx+1:02d} {label}", fill=(230, 220, 200))
    out = OUT / "contact_sheet.jpg"
    sheet.save(out, quality=90)
    print(f"wrote {out}")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)

    d = sub.add_parser("discover")
    d.add_argument("--search", action="store_true", help="also run yt-dlp YouTube searches")
    d.add_argument("--per-query", type=int, default=1)
    d.set_defaults(func=discover)

    dl = sub.add_parser("download")
    dl.add_argument("--limit", type=int, default=0, help="0 means all selected catalog videos")
    dl.add_argument("--offset", type=int, default=0)
    dl.add_argument("--video-id", action="append", help="download only this YouTube video id; can be repeated")
    dl.add_argument(
        "--format",
        default="best[height<=480]/best[height<=720]/best",
        help="yt-dlp format selector; default keeps local research copies modest",
    )
    dl.add_argument("--max-filesize", help="yt-dlp size guard, e.g. 250M")
    dl.add_argument("--timeout", type=int, default=900)
    dl.add_argument("--dry-run", action="store_true")
    dl.set_defaults(func=download)

    e = sub.add_parser("extract")
    e.add_argument("--limit", type=int, default=5, help="0 means all selected catalog videos")
    e.add_argument("--offset", type=int, default=0)
    e.add_argument("--video-id", action="append", help="extract only this YouTube video id; can be repeated")
    e.add_argument("--frames-per-video", type=int, default=4)
    e.add_argument(
        "--seconds",
        type=float,
        action="append",
        help="extract an exact second; can be repeated and overrides --frames-per-video",
    )
    e.add_argument("--width", type=int, default=768)
    e.add_argument("--append", action="store_true", help="keep existing frames instead of clearing the frame folder")
    e.add_argument("--stream", action="store_true", help="ignore local downloads and stream via yt-dlp media URLs")
    e.set_defaults(func=extract)

    c = sub.add_parser("contact-sheet")
    c.add_argument("--columns", type=int, default=3)
    c.add_argument("--thumb-width", type=int, default=320)
    c.add_argument("--thumb-height", type=int, default=180)
    c.set_defaults(func=contact_sheet)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
