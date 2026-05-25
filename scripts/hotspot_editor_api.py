#!/usr/bin/env python3
"""Local write API for the hotspot editor.

The browser editor cannot write project files by itself. This localhost-only
helper accepts the current editor geometry and applies it to the Rust scene
definitions plus the editor's own defaults.
"""

from __future__ import annotations

import argparse
import json
import mimetypes
import re
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse


ROOT = Path(__file__).resolve().parents[1]
RUST_FILE = ROOT / "src" / "main.rs"
EDITOR_FILE = ROOT / "web" / "hotspot-editor.html"
VIDEO_ROOM_DIR = ROOT / "tmp" / "klixx-video-rooms"
VIDEOS_DIR = VIDEO_ROOM_DIR / "videos"
CATALOG_FILE = VIDEO_ROOM_DIR / "catalog.json"
FRAMES_FILE = VIDEO_ROOM_DIR / "frames.json"
FRAME_SELECTIONS_FILE = VIDEO_ROOM_DIR / "frame_selections.json"
ASSET_SELECTED_FRAMES_FILE = ROOT / "assets" / "selected_video_frames.json"
MAX_BODY = 10_000_000
VIDEO_EXTENSIONS = {".mp4", ".webm", ".mkv", ".mov"}
HOTSPOT_KINDS = {"Character", "Pickup", "Prop", "Exit"}
NEW_HOTSPOT_SOURCE = "editor_new_hotspot"
DEFAULT_NEW_HOTSPOT_LOOK = "Noch nicht beschrieben."
DEFAULT_NEW_HOTSPOT_INSPECT = "Noch nicht beschrieben."
FRAME_PICK_TIMEOUT_SECONDS = 25
WRITE_LOCK = threading.RLock()


def rounded(value: Any) -> float:
    return round(float(value), 2)


def rust_float(value: Any) -> str:
    text = f"{rounded(value):.2f}".rstrip("0").rstrip(".")
    return f"{text}.0" if "." not in text else text


def js_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def sanitize_pct(pct: dict[str, Any]) -> dict[str, float]:
    x = max(0.0, min(99.5, rounded(pct.get("x", 0))))
    y = max(0.0, min(99.5, rounded(pct.get("y", 0))))
    w = max(0.5, min(100.0 - x, rounded(pct.get("w", 0.5))))
    h = max(0.5, min(100.0 - y, rounded(pct.get("h", 0.5))))
    return {"x": rounded(x), "y": rounded(y), "w": rounded(w), "h": rounded(h)}


def sanitize_point(point: dict[str, Any]) -> dict[str, float]:
    return {
        "x": max(0.0, min(100.0, rounded(point.get("x", 0)))),
        "y": max(0.0, min(100.0, rounded(point.get("y", 0)))),
    }


def rect_polygon(pct: dict[str, float]) -> list[dict[str, float]]:
    right = rounded(pct["x"] + pct["w"])
    bottom = rounded(pct["y"] + pct["h"])
    return [
        {"x": pct["x"], "y": pct["y"]},
        {"x": right, "y": pct["y"]},
        {"x": right, "y": bottom},
        {"x": pct["x"], "y": bottom},
    ]


def sanitize_polygon(raw_polygon: Any, pct: dict[str, float]) -> list[dict[str, float]]:
    if not isinstance(raw_polygon, list):
        return rect_polygon(pct)

    points = [
        sanitize_point(point)
        for point in raw_polygon
        if isinstance(point, dict)
    ]
    if len(points) < 3:
        return rect_polygon(pct)
    return points


def sanitize_kind(raw_kind: Any) -> str:
    kind = str(raw_kind or "Prop").strip()
    return kind if kind in HOTSPOT_KINDS else "Prop"


def sanitize_optional_id(raw_value: Any) -> str | None:
    if raw_value is None:
        return None
    value = str(raw_value).strip()
    if not value:
        return None
    if not re.fullmatch(r"[a-zA-Z0-9_]+", value):
        return None
    return value


def is_new_hotspot(hotspot: dict[str, Any]) -> bool:
    hotspot_id = hotspot.get("id", "")
    return (
        hotspot.get("source") == NEW_HOTSPOT_SOURCE
        or hotspot_id.startswith("draft_")
        or hotspot_id.startswith("new_")
    )


def sanitize_scenes(raw_scenes: Any) -> list[dict[str, Any]]:
    if not isinstance(raw_scenes, list) or not raw_scenes:
        raise ValueError("scenes must be a non-empty list")

    scenes: list[dict[str, Any]] = []
    for raw_scene in raw_scenes:
        if not isinstance(raw_scene, dict):
            raise ValueError("each scene must be an object")
        scene_id = str(raw_scene.get("id", "")).strip()
        if not re.fullmatch(r"[a-zA-Z0-9_]+", scene_id):
            raise ValueError(f"invalid scene id: {scene_id!r}")

        hotspots = []
        for raw_hotspot in raw_scene.get("hotspots", []):
            if not isinstance(raw_hotspot, dict):
                raise ValueError(f"{scene_id}: hotspot must be an object")
            hotspot_id = str(raw_hotspot.get("id", "")).strip()
            if not re.fullmatch(r"[a-zA-Z0-9_]+", hotspot_id):
                raise ValueError(f"{scene_id}: invalid hotspot id {hotspot_id!r}")
            pct = sanitize_pct(raw_hotspot.get("pct", {}))
            hotspot = {
                "id": hotspot_id,
                "name": str(raw_hotspot.get("name", hotspot_id)).strip() or hotspot_id,
                "kind": sanitize_kind(raw_hotspot.get("kind")),
                "pct": pct,
                "polygon": sanitize_polygon(raw_hotspot.get("polygon"), pct),
            }
            source = sanitize_optional_id(raw_hotspot.get("source"))
            if source:
                hotspot["source"] = source
            talk_id = sanitize_optional_id(raw_hotspot.get("talk_id"))
            if talk_id:
                hotspot["talk_id"] = talk_id
            for key in ("look", "inspect"):
                if raw_hotspot.get(key):
                    hotspot[key] = str(raw_hotspot[key]).strip()
            hotspots.append(hotspot)

        walkable = [sanitize_point(point) for point in raw_scene.get("walkable", [])]
        scenes.append(
            {
                "id": scene_id,
                "name": str(raw_scene.get("name", scene_id)),
                "zone": str(raw_scene.get("zone", "")),
                "token": [
                    max(0.0, min(100.0, rounded((raw_scene.get("token") or [50, 82])[0]))),
                    max(0.0, min(100.0, rounded((raw_scene.get("token") or [50, 82])[1]))),
                ],
                "hotspots": hotspots,
                "walkable": walkable,
            }
        )

    return scenes


def scene_bindings(rust: str) -> dict[str, dict[str, str]]:
    bindings: dict[str, dict[str, str]] = {}
    for match in re.finditer(r"SceneMeta\s*\{(?P<body>.*?)\n\s*\}", rust, re.S):
        body = match.group("body")
        scene_id = re.search(r'id:\s*"([^"]+)"', body)
        walkable = re.search(r"walkable:\s*([A-Z0-9_]+)", body)
        hotspots = re.search(r"hotspots:\s*([A-Z0-9_]+)", body)
        if scene_id and walkable and hotspots:
            bindings[scene_id.group(1)] = {
                "walkable": walkable.group(1),
                "hotspots": hotspots.group(1),
            }
    return bindings


def pct_call(pct: dict[str, float]) -> str:
    return (
        f"pct({rust_float(pct['x'])}, {rust_float(pct['y'])}, "
        f"{rust_float(pct['w'])}, {rust_float(pct['h'])})"
    )


def rust_hotspot_spec(scene_id: str, hotspot: dict[str, Any]) -> str:
    talk_id = hotspot.get("talk_id")
    talk = f"Some({js_string(talk_id)})" if talk_id else "None"
    look = hotspot.get("look") or DEFAULT_NEW_HOTSPOT_LOOK
    inspect = hotspot.get("inspect") or DEFAULT_NEW_HOTSPOT_INSPECT
    return "\n".join(
        [
            f"    // NEW_HOTSPOT {scene_id}/{hotspot['id']}",
            "    HotspotSpec {",
            f"        id: {js_string(hotspot['id'])},",
            f"        name: {js_string(hotspot['name'])},",
            f"        pct: {pct_call(hotspot['pct'])},",
            f"        kind: HotspotKind::{sanitize_kind(hotspot.get('kind'))},",
            f"        look: {js_string(look)},",
            f"        inspect: {js_string(inspect)},",
            f"        talk_id: {talk},",
            "    },",
        ]
    )


def point_call(point: dict[str, float]) -> str:
    return f"({rust_float(point['x'])}, {rust_float(point['y'])})"


def walkable_const(name: str, points: list[dict[str, float]]) -> str:
    if len(points) <= 4:
        joined = ", ".join(point_call(point) for point in points)
        return f"const {name}: &[(f32, f32)] = &[{joined}];"

    lines = [f"const {name}: &[(f32, f32)] = &["]
    lines.extend(f"    {point_call(point)}," for point in points)
    lines.append("];")
    return "\n".join(lines)


def hotspot_polygon_const(scenes: list[dict[str, Any]]) -> str:
    lines = ["const HOTSPOT_POLYGONS: &[HotspotPolygonSpec] = &["]
    for current_scene in scenes:
        for hotspot in current_scene["hotspots"]:
            points = hotspot.get("polygon") or rect_polygon(hotspot["pct"])
            joined = ", ".join(point_call(point) for point in points)
            lines.append(
                "    HotspotPolygonSpec { "
                f"scene_id: {js_string(current_scene['id'])}, "
                f"hotspot_id: {js_string(hotspot['id'])}, "
                f"points: &[{joined}] "
                "},"
            )
    lines.append("];")
    return "\n".join(lines)


def update_rust(rust: str, scenes: list[dict[str, Any]]) -> tuple[str, list[str]]:
    bindings = scene_bindings(rust)
    warnings: list[str] = []
    next_rust = rust

    for current_scene in scenes:
        binding = bindings.get(current_scene["id"])
        if not binding:
            warnings.append(f"Rust scene not found: {current_scene['id']}")
            continue

        walkable_name = binding["walkable"]
        walkable_pattern = re.compile(
            rf"const\s+{re.escape(walkable_name)}:\s*&\[\(f32,\s*f32\)\]\s*=\s*&\[.*?\];",
            re.S,
        )
        next_rust, count = walkable_pattern.subn(
            walkable_const(walkable_name, current_scene["walkable"]),
            next_rust,
            count=1,
        )
        if count == 0:
            warnings.append(f"Rust walkable const not found: {walkable_name}")

        hotspots_name = binding["hotspots"]
        block_pattern = re.compile(
            rf"(const\s+{re.escape(hotspots_name)}:\s*&\[HotspotSpec\]\s*=\s*&\[)"
            rf"(?P<body>.*?)(\n\];)",
            re.S,
        )
        block_match = block_pattern.search(next_rust)
        if not block_match:
            warnings.append(f"Rust hotspot const not found: {hotspots_name}")
            continue

        body = block_match.group("body")
        for hotspot in current_scene["hotspots"]:
            if is_new_hotspot(hotspot):
                hotspot_block_pattern = re.compile(
                    rf"\n\s*(?:// NEW_HOTSPOT [^\n]*\n)?\s*HotspotSpec\s*\{{"
                    rf'(?:(?!\n\s*HotspotSpec\s*\{{).)*?id:\s*"{re.escape(hotspot["id"])}"\s*,'
                    rf"(?:(?!\n\s*HotspotSpec\s*\{{).)*?\n\s*\}},",
                    re.S,
                )
                body, count = hotspot_block_pattern.subn(
                    "\n" + rust_hotspot_spec(current_scene["id"], hotspot),
                    body,
                    count=1,
                )
                if count == 0:
                    body = body.rstrip() + "\n" + rust_hotspot_spec(current_scene["id"], hotspot)
                continue

            hotspot_pattern = re.compile(
                rf'(id:\s*"{re.escape(hotspot["id"])}"\s*,'
                rf'(?:(?!\n\s*HotspotSpec\s*\{{).)*?pct:\s*)pct\([^)]*\)',
                re.S,
            )
            body, count = hotspot_pattern.subn(
                rf"\1{pct_call(hotspot['pct'])}",
                body,
                count=1,
            )
            if count == 0:
                warnings.append(
                    f"Rust hotspot not found: {current_scene['id']}/{hotspot['id']}"
                )

        next_rust = (
            next_rust[: block_match.start("body")]
            + body
            + next_rust[block_match.end("body") :]
        )

    polygon_pattern = re.compile(
        r"const\s+HOTSPOT_POLYGONS:\s*&\[HotspotPolygonSpec\]\s*=\s*&\[.*?\];",
        re.S,
    )
    next_rust, count = polygon_pattern.subn(
        hotspot_polygon_const(scenes),
        next_rust,
        count=1,
    )
    if count == 0:
        warnings.append("Rust HOTSPOT_POLYGONS const not found")

    return next_rust, warnings


def js_initial_scenes(scenes: list[dict[str, Any]]) -> str:
    lines = ["const initialScenes = ["]
    for current_scene in scenes:
        lines.append("  {")
        lines.append(f"    id: {js_string(current_scene['id'])},")
        lines.append(f"    name: {js_string(current_scene['name'])},")
        lines.append(f"    zone: {js_string(current_scene['zone'])},")
        lines.append(
            "    token: "
            f"[{rust_float(current_scene['token'][0])}, {rust_float(current_scene['token'][1])}],"
        )
        lines.append("    hotspots: [")
        for hotspot in current_scene["hotspots"]:
            pct = hotspot["pct"]
            polygon = hotspot.get("polygon") or rect_polygon(pct)
            lines.append("      {")
            lines.append(f"        id: {js_string(hotspot['id'])},")
            lines.append(f"        name: {js_string(hotspot['name'])},")
            lines.append(f"        kind: {js_string(hotspot['kind'])},")
            if hotspot.get("source"):
                lines.append(f"        source: {js_string(hotspot['source'])},")
            if hotspot.get("look"):
                lines.append(f"        look: {js_string(hotspot['look'])},")
            if hotspot.get("inspect"):
                lines.append(f"        inspect: {js_string(hotspot['inspect'])},")
            if hotspot.get("talk_id"):
                lines.append(f"        talk_id: {js_string(hotspot['talk_id'])},")
            lines.append(
                "        pct: { "
                f"x: {rust_float(pct['x'])}, y: {rust_float(pct['y'])}, "
                f"w: {rust_float(pct['w'])}, h: {rust_float(pct['h'])} "
                "},"
            )
            lines.append("        polygon: [")
            for point in polygon:
                lines.append(
                    f"          {{ x: {rust_float(point['x'])}, y: {rust_float(point['y'])} }},"
                )
            lines.append("        ],")
            lines.append("      },")
        lines.append("    ],")
        lines.append("  },")
    lines.append("];")
    return "\n".join(lines)


def js_walkable(scenes: list[dict[str, Any]]) -> str:
    lines = ["const SCENE_WALKABLE = {"]
    for current_scene in scenes:
        lines.append(f"  {js_string(current_scene['id'])}: [")
        for point in current_scene["walkable"]:
            lines.append(
                f"    {{ x: {rust_float(point['x'])}, y: {rust_float(point['y'])} }},"
            )
        lines.append("  ],")
    lines.append("};")
    return "\n".join(lines)


def update_editor(editor: str, scenes: list[dict[str, Any]]) -> tuple[str, list[str]]:
    warnings: list[str] = []
    next_editor, count = re.subn(
        r"const initialScenes = \[.*?\];\n\nconst DEFAULT_WALKABLE",
        js_initial_scenes(scenes) + "\n\nconst DEFAULT_WALKABLE",
        editor,
        count=1,
        flags=re.S,
    )
    if count == 0:
        warnings.append("Editor initialScenes block not found")

    next_editor, count = re.subn(
        r"const SCENE_WALKABLE = \{.*?\};\n\nfunction deepClone",
        js_walkable(scenes) + "\n\nfunction deepClone",
        next_editor,
        count=1,
        flags=re.S,
    )
    if count == 0:
        warnings.append("Editor SCENE_WALKABLE block not found")

    return next_editor, warnings


def write_if_changed(path: Path, content: str) -> bool:
    old = path.read_text(encoding="utf-8")
    if old == content:
        return False
    path.write_text(content, encoding="utf-8")
    return True


def apply_definitions(scenes: list[dict[str, Any]], dry_run: bool) -> dict[str, Any]:
    rust = RUST_FILE.read_text(encoding="utf-8")
    editor = EDITOR_FILE.read_text(encoding="utf-8")
    next_rust, rust_warnings = update_rust(rust, scenes)
    next_editor, editor_warnings = update_editor(editor, scenes)

    changed: list[str] = []
    if next_rust != rust:
        changed.append(str(RUST_FILE.relative_to(ROOT)))
    if next_editor != editor:
        changed.append(str(EDITOR_FILE.relative_to(ROOT)))

    if not dry_run:
        write_if_changed(RUST_FILE, next_rust)
        write_if_changed(EDITOR_FILE, next_editor)

    return {
        "ok": True,
        "dry_run": dry_run,
        "changed_files": changed,
        "warnings": rust_warnings + editor_warnings,
    }


def safe_project_path(raw_path: str) -> Path:
    path = (ROOT / raw_path).resolve()
    if not path.is_relative_to(ROOT):
        raise ValueError("path escapes project root")
    return path


def load_json(path: Path, fallback: Any) -> Any:
    if not path.exists():
        return fallback
    return json.loads(path.read_text(encoding="utf-8"))


def video_id_from_url(url: str) -> str:
    parsed = urlparse(url)
    if parsed.hostname and "youtu.be" in parsed.hostname:
        return parsed.path.strip("/")
    query_id = parse_qs(parsed.query).get("v", [""])[0]
    return query_id or ""


def slug(text: str) -> str:
    cleaned = re.sub(r"[^a-zA-Z0-9]+", "_", text.lower()).strip("_")
    return cleaned[:70] or "video"


def video_id_from_path(path: Path) -> str:
    return path.name.split("_", 1)[0]


def catalog_by_id() -> dict[str, dict[str, Any]]:
    catalog = load_json(CATALOG_FILE, {"videos": []})
    result: dict[str, dict[str, Any]] = {}
    for entry in catalog.get("videos", []):
        url = entry.get("webpage_url") or entry.get("url") or ""
        vid = video_id_from_url(url)
        if vid:
            result[vid] = entry
    return result


def local_video_files() -> list[Path]:
    if not VIDEOS_DIR.exists():
        return []
    return sorted(
        path
        for path in VIDEOS_DIR.iterdir()
        if path.is_file() and path.suffix.lower() in VIDEO_EXTENSIONS
    )


def video_entries() -> list[dict[str, Any]]:
    catalog = catalog_by_id()
    downloads = load_json(VIDEO_ROOM_DIR / "downloads.json", [])
    by_path: dict[str, dict[str, Any]] = {}

    for download in downloads:
        vid = str(download.get("video_id") or "")
        for raw_file in download.get("files", []):
            path = safe_project_path(str(raw_file))
            if not path.exists() or path.suffix.lower() not in VIDEO_EXTENSIONS:
                continue
            info = catalog.get(vid, {})
            by_path[str(path.relative_to(ROOT))] = {
                "video_id": vid or video_id_from_path(path),
                "title": download.get("title") or info.get("title") or path.stem,
                "url": download.get("url") or info.get("webpage_url") or info.get("url") or "",
                "duration": info.get("duration"),
                "path": str(path.relative_to(ROOT)),
                "size": path.stat().st_size,
            }

    for path in local_video_files():
        rel = str(path.relative_to(ROOT))
        if rel in by_path:
            continue
        vid = video_id_from_path(path)
        info = catalog.get(vid, {})
        by_path[rel] = {
            "video_id": vid,
            "title": info.get("title") or path.stem,
            "url": info.get("webpage_url") or info.get("url") or "",
            "duration": info.get("duration"),
            "path": rel,
            "size": path.stat().st_size,
        }

    return sorted(by_path.values(), key=lambda item: (item["title"].lower(), item["video_id"]))


def video_index() -> dict[str, Any]:
    return {
        "ok": True,
        "videos_dir": str(VIDEOS_DIR.relative_to(ROOT)),
        "videos": video_entries(),
    }


def frame_entries() -> list[dict[str, Any]]:
    seen: set[str] = set()
    normalized: list[dict[str, Any]] = []

    with WRITE_LOCK:
        frames = load_json(FRAMES_FILE, [])
        for frame in frames:
            path = str(frame.get("path", ""))
            if not path or path in seen:
                continue
            seen.add(path)
            file_path = safe_project_path(path)
            item = dict(frame)
            item["path"] = path
            item["exists"] = file_path.exists()
            if file_path.exists():
                item["mtime"] = file_path.stat().st_mtime
            normalized.append(item)

        for file_path in sorted((VIDEO_ROOM_DIR / "frames").glob("*/*.[jp][pn][g]")):
            rel = str(file_path.relative_to(ROOT))
            if rel in seen:
                continue
            seen.add(rel)
            video_id = file_path.parent.name.split("_", 1)[0]
            match = re.search(r"_(\d+)s\.", file_path.name)
            normalized.append(
                {
                    "video_id": video_id,
                    "title": file_path.parent.name,
                    "url": "",
                    "seconds": int(match.group(1)) if match else None,
                    "path": rel,
                    "exists": True,
                    "mtime": file_path.stat().st_mtime,
                }
            )

    return normalized


def frame_index() -> dict[str, Any]:
    catalog = load_json(CATALOG_FILE, {"videos": []})
    with WRITE_LOCK:
        frames = frame_entries()
        selections = load_json(FRAME_SELECTIONS_FILE, {"frames": {}})
    return {
        "ok": True,
        "catalog_path": str(CATALOG_FILE.relative_to(ROOT)),
        "frames_path": str(FRAMES_FILE.relative_to(ROOT)),
        "selections_path": str(FRAME_SELECTIONS_FILE.relative_to(ROOT)),
        "videos": catalog.get("videos", []),
        "frames": frames,
        "selections": selections.get("frames", selections if isinstance(selections, dict) else {}),
    }


def sanitize_frame_selection(raw: dict[str, Any]) -> dict[str, Any]:
    path = str(raw.get("path", ""))
    file_path = safe_project_path(path)
    if not file_path.is_relative_to(VIDEO_ROOM_DIR.resolve()):
        raise ValueError(f"frame path outside tmp frame workspace: {path}")

    status = str(raw.get("status", "unreviewed"))
    if status not in {"unreviewed", "selected", "rejected"}:
        status = "unreviewed"
    tags = raw.get("tags") if isinstance(raw.get("tags"), list) else []
    return {
        "path": path,
        "status": status,
        "walkable": bool(raw.get("walkable")),
        "scene_id": str(raw.get("scene_id", ""))[:80],
        "notes": str(raw.get("notes", ""))[:2000],
        "tags": [str(tag)[:40] for tag in tags[:20]],
    }


def save_frame_selections(payload: dict[str, Any], dry_run: bool) -> dict[str, Any]:
    raw_frames = payload.get("frames", {})
    if isinstance(raw_frames, list):
        entries = [sanitize_frame_selection(item) for item in raw_frames]
    elif isinstance(raw_frames, dict):
        entries = [sanitize_frame_selection({"path": path, **value}) for path, value in raw_frames.items()]
    else:
        raise ValueError("frames must be an object or list")

    frames_by_path = {entry["path"]: entry for entry in entries}
    selected = [entry for entry in entries if entry["status"] == "selected"]
    output = {
        "generated_by": "scripts/hotspot_editor_api.py",
        "frames": frames_by_path,
        "selected": selected,
    }

    changed_files = [
        str(FRAME_SELECTIONS_FILE.relative_to(ROOT)),
        str(ASSET_SELECTED_FRAMES_FILE.relative_to(ROOT)),
    ]
    if not dry_run:
        with WRITE_LOCK:
            FRAME_SELECTIONS_FILE.parent.mkdir(parents=True, exist_ok=True)
            FRAME_SELECTIONS_FILE.write_text(json.dumps(output, ensure_ascii=False, indent=2), encoding="utf-8")
            ASSET_SELECTED_FRAMES_FILE.parent.mkdir(parents=True, exist_ok=True)
            ASSET_SELECTED_FRAMES_FILE.write_text(
                json.dumps({"selected": selected}, ensure_ascii=False, indent=2),
                encoding="utf-8",
            )

    return {
        "ok": True,
        "dry_run": dry_run,
        "selected_count": len(selected),
        "changed_files": changed_files,
    }


def write_frame_selection(entry: dict[str, Any]) -> None:
    current = load_json(FRAME_SELECTIONS_FILE, {"frames": {}})
    frames = current.get("frames", {}) if isinstance(current, dict) else {}
    sanitized = sanitize_frame_selection(entry)
    frames[sanitized["path"]] = sanitized
    selected = [item for item in frames.values() if item.get("status") == "selected"]
    output = {
        "generated_by": "scripts/hotspot_editor_api.py",
        "frames": frames,
        "selected": selected,
    }
    FRAME_SELECTIONS_FILE.parent.mkdir(parents=True, exist_ok=True)
    FRAME_SELECTIONS_FILE.write_text(json.dumps(output, ensure_ascii=False, indent=2), encoding="utf-8")
    ASSET_SELECTED_FRAMES_FILE.parent.mkdir(parents=True, exist_ok=True)
    ASSET_SELECTED_FRAMES_FILE.write_text(
        json.dumps({"selected": selected}, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )


def upsert_frame_entry(entry: dict[str, Any]) -> None:
    frames = load_json(FRAMES_FILE, [])
    next_frames = [frame for frame in frames if frame.get("path") != entry["path"]]
    next_frames.append(entry)
    FRAMES_FILE.parent.mkdir(parents=True, exist_ok=True)
    FRAMES_FILE.write_text(json.dumps(next_frames, ensure_ascii=False, indent=2), encoding="utf-8")


def pick_frame(payload: dict[str, Any]) -> dict[str, Any]:
    raw_video_path = str(payload.get("video_path") or "")
    video_path = safe_project_path(raw_video_path)
    if not video_path.is_relative_to(VIDEOS_DIR.resolve()):
        raise ValueError("video path outside videos directory")
    if not video_path.exists() or video_path.suffix.lower() not in VIDEO_EXTENSIONS:
        raise ValueError("video file not found")

    seconds = max(0.0, float(payload.get("seconds", 0)))
    vid = str(payload.get("video_id") or video_id_from_path(video_path))
    title = str(payload.get("title") or video_path.stem)
    frame_dir = VIDEO_ROOM_DIR / "frames" / f"{vid}_{slug(title)}"
    filename = f"pick_{int(seconds * 1000):09d}ms.jpg"
    out = frame_dir / filename
    rel = str(out.relative_to(ROOT))
    dry_run = bool(payload.get("dry_run"))

    if not dry_run:
        frame_dir.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                f"{seconds:.3f}",
                "-i",
                str(video_path),
                "-frames:v",
                "1",
                "-vf",
                "scale=768:-1",
                "-y",
                str(out),
            ],
            cwd=ROOT,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=FRAME_PICK_TIMEOUT_SECONDS,
        )

    frame_entry = {
        "video_id": vid,
        "title": title,
        "url": str(payload.get("url") or ""),
        "source": "video-frame-picker",
        "episode_hint": str(payload.get("episode_hint") or ""),
        "seconds": seconds,
        "path": rel,
    }
    selection_entry = {
        "path": rel,
        "status": str(payload.get("status") or "selected"),
        "walkable": bool(payload.get("walkable", True)),
        "scene_id": str(payload.get("scene_id") or ""),
        "notes": str(payload.get("notes") or ""),
        "tags": payload.get("tags") if isinstance(payload.get("tags"), list) else [],
    }

    if not dry_run:
        with WRITE_LOCK:
            upsert_frame_entry(frame_entry)
            write_frame_selection(selection_entry)

    return {
        "ok": True,
        "dry_run": dry_run,
        "frame": frame_entry,
        "selection": sanitize_frame_selection(selection_entry),
        "changed_files": [
            rel,
            str(FRAMES_FILE.relative_to(ROOT)),
            str(FRAME_SELECTIONS_FILE.relative_to(ROOT)),
            str(ASSET_SELECTED_FRAMES_FILE.relative_to(ROOT)),
        ],
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt: str, *args: Any) -> None:
        print(f"{self.address_string()} - {fmt % args}")

    def send_json(self, status: int, payload: dict[str, Any]) -> None:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.end_headers()
        self.wfile.write(body)

    def send_file(self, path: Path, *, allow_range: bool = False) -> None:
        if not path.exists() or not path.is_file():
            self.send_json(404, {"ok": False, "error": "file not found"})
            return
        file_size = path.stat().st_size
        content_type = mimetypes.guess_type(path.name)[0] or "application/octet-stream"
        start = 0
        end = file_size - 1
        status = 200

        if allow_range:
            range_header = self.headers.get("Range", "")
            match = re.fullmatch(r"bytes=(\d*)-(\d*)", range_header)
            if match:
                if match.group(1):
                    start = int(match.group(1))
                if match.group(2):
                    end = int(match.group(2))
                end = min(end, file_size - 1)
                if start > end:
                    self.send_response(416)
                    self.send_header("Content-Range", f"bytes */{file_size}")
                    self.send_header("Access-Control-Allow-Origin", "*")
                    self.end_headers()
                    return
                status = 206

        length = end - start + 1
        try:
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(length))
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Cache-Control", "no-store")
            if allow_range:
                self.send_header("Accept-Ranges", "bytes")
            if status == 206:
                self.send_header("Content-Range", f"bytes {start}-{end}/{file_size}")
            self.end_headers()
            with path.open("rb") as handle:
                handle.seek(start)
                remaining = length
                while remaining > 0:
                    chunk = handle.read(min(1024 * 1024, remaining))
                    if not chunk:
                        break
                    self.wfile.write(chunk)
                    remaining -= len(chunk)
        except (BrokenPipeError, ConnectionResetError):
            return

    def do_OPTIONS(self) -> None:
        self.send_json(204, {})

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/health":
            self.send_json(200, {"ok": True})
            return
        if parsed.path == "/frame-index":
            self.send_json(200, frame_index())
            return
        if parsed.path == "/video-index":
            self.send_json(200, video_index())
            return
        if parsed.path == "/frame-image":
            try:
                raw_path = parse_qs(parsed.query).get("path", [""])[0]
                path = safe_project_path(raw_path)
                if not path.is_relative_to((VIDEO_ROOM_DIR / "frames").resolve()):
                    raise ValueError("frame image path outside frames directory")
                self.send_file(path)
            except Exception as error:  # noqa: BLE001 - local dev API.
                self.send_json(400, {"ok": False, "error": str(error)})
            return
        if parsed.path == "/video-file":
            try:
                raw_path = parse_qs(parsed.query).get("path", [""])[0]
                path = safe_project_path(raw_path)
                if not path.is_relative_to(VIDEOS_DIR.resolve()):
                    raise ValueError("video path outside videos directory")
                self.send_file(path, allow_range=True)
            except Exception as error:  # noqa: BLE001 - local dev API.
                self.send_json(400, {"ok": False, "error": str(error)})
            return
        self.send_json(404, {"ok": False, "error": "not found"})

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path not in {"/apply-scene-definitions", "/frame-selections", "/pick-frame"}:
            self.send_json(404, {"ok": False, "error": "not found"})
            return

        try:
            length = int(self.headers.get("Content-Length", "0"))
            if length <= 0 or length > MAX_BODY:
                raise ValueError("invalid request size")
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            if parsed.path == "/apply-scene-definitions":
                scenes = sanitize_scenes(payload.get("scenes"))
                result = apply_definitions(scenes, dry_run=bool(payload.get("dry_run")))
            elif parsed.path == "/frame-selections":
                result = save_frame_selections(payload, dry_run=bool(payload.get("dry_run")))
            else:
                result = pick_frame(payload)
            self.send_json(200, result)
        except Exception as error:  # noqa: BLE001 - return validation details to local UI.
            self.send_json(400, {"ok": False, "error": str(error)})


class LocalThreadingHTTPServer(ThreadingHTTPServer):
    daemon_threads = True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8082)
    args = parser.parse_args()

    server = LocalThreadingHTTPServer((args.host, args.port), Handler)
    print(f"Hotspot editor write API: http://{args.host}:{args.port}/")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
