#!/usr/bin/env python3
"""Maintain generated support assets for the video-room slice."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]


def c(hex_value: str) -> tuple[int, int, int, int]:
    hex_value = hex_value.strip("#")
    return (
        int(hex_value[0:2], 16),
        int(hex_value[2:4], 16),
        int(hex_value[4:6], 16),
        255,
    )


def extend_inventory_icons() -> None:
    path = ROOT / "assets" / "sprites" / "inventory_icons.png"
    img = Image.open(path).convert("RGBA")
    if img.width < 512:
        expanded = Image.new("RGBA", (512, 256), (0, 0, 0, 0))
        expanded.paste(img, (0, 0))
        img = expanded
    draw = ImageDraw.Draw(img)
    x, y = 256, 128
    draw.rectangle((x, y, x + 127, y + 127), fill=(0, 0, 0, 0))
    draw.polygon(
        [(x + 34, y + 24), (x + 93, y + 18), (x + 100, y + 98), (x + 42, y + 107)],
        fill=c("d8d0ba"),
        outline=c("241f1c"),
    )
    for yy in (43, 56, 69, 82):
        draw.line((x + 45, y + yy, x + 86, y + yy - 4), fill=c("6a5142"), width=3)
    draw.line((x + 59, y + 24, x + 79, y + 103), fill=c("9e7b4b"), width=2)
    draw.ellipse((x + 78, y + 78, x + 108, y + 108), fill=c("3b5861"), outline=c("d4b36a"), width=3)
    x, y = 384, 0
    draw.rectangle((x, y, x + 127, y + 127), fill=(0, 0, 0, 0))
    draw.polygon(
        [(x + 27, y + 28), (x + 98, y + 18), (x + 108, y + 88), (x + 39, y + 104)],
        fill=c("232d2d"),
        outline=c("d5b45f"),
    )
    draw.line((x + 35, y + 70, x + 93, y + 62), fill=c("d5b45f"), width=5)
    draw.polygon([(x + 93, y + 62), (x + 82, y + 55), (x + 84, y + 70)], fill=c("d5b45f"))
    for sx, sy in [(x + 44, y + 34), (x + 58, y + 31), (x + 73, y + 28)]:
        draw.rectangle((sx, sy, sx + 8, sy + 30), fill=c("b92f2b"))
        draw.rectangle((sx, sy + 7, sx + 8, sy + 10), fill=c("eee3c1"))
        draw.rectangle((sx, sy + 18, sx + 8, sy + 21), fill=c("eee3c1"))
    draw.rectangle((x + 39, y + 91, x + 91, y + 99), fill=c("ded7be"))
    draw.line((x + 47, y + 95, x + 83, y + 94), fill=c("6a5849"), width=2)
    img.save(path)
    print(path)


def main() -> None:
    extend_inventory_icons()


if __name__ == "__main__":
    main()
