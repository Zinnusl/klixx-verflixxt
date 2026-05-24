#!/usr/bin/env python3
"""Generate local fallback assets for the first video-room slice."""

from __future__ import annotations

import math
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


def draw_sewer_room() -> None:
    scale = 4
    low = Image.new("RGBA", (310, 120), c("151514"))
    draw = ImageDraw.Draw(low)

    for y in range(120):
        shade = int(18 + y * 0.26)
        draw.line((0, y, 310, y), fill=(shade, shade - 3, shade - 8, 255))

    center = (155, 55)
    for i in range(17, 0, -1):
        rx = 145 - i * 5
        ry = 72 - i * 3
        if rx <= 8 or ry <= 6:
            continue
        tone = 52 + i * 7
        ring = (tone, max(30, tone - 24), max(24, tone - 33), 255)
        box = (center[0] - rx, center[1] - ry, center[0] + rx, center[1] + ry)
        draw.ellipse(box, outline=ring, width=2)

    for i in range(0, 70, 5):
        color = (78 + i, 59 + i // 2, 47 + i // 3, 255)
        draw.arc((15 + i, -10 + i // 3, 295 - i, 112 - i // 5), 180, 350, fill=color, width=1)

    draw.ellipse((96, 18, 214, 92), fill=c("181513"))
    draw.ellipse((113, 30, 197, 83), fill=c("0e0d0c"))
    draw.ellipse((133, 41, 177, 70), fill=c("080807"))

    draw.polygon([(0, 84), (310, 84), (310, 120), (0, 120)], fill=c("20201d"))
    for y in range(86, 120, 6):
        draw.line((0, y, 310, y + 4), fill=c("3c342e"))
    for x in range(-20, 330, 32):
        draw.line((x, 84, x - 26, 120), fill=c("171615"))

    draw.polygon([(168, 84), (197, 90), (209, 103), (156, 99)], fill=c("433629"))
    draw.rectangle((58, 65, 83, 83), fill=c("222321"))
    draw.line((65, 65, 60, 92), fill=c("a78349"), width=2)
    draw.line((76, 65, 87, 92), fill=c("a78349"), width=2)
    draw.line((66, 70, 86, 70), fill=c("d2b36f"), width=2)
    draw.rectangle((69, 56, 75, 66), fill=c("36444a"))
    draw.line((74, 58, 93, 48), fill=c("8d7f5a"), width=1)

    hatch = (224, 48, 264, 86)
    draw.ellipse(hatch, fill=c("211e1b"), outline=c("bc8d42"), width=2)
    draw.arc((229, 52, 259, 82), 200, 520, fill=c("62513b"), width=2)
    draw.rectangle((241, 64, 248, 70), fill=c("d0b56f"))

    draw.line((129, 92, 167, 92), fill=c("cc9a45"), width=2)
    draw.polygon([(168, 92), (159, 88), (159, 96)], fill=c("cc9a45"))
    draw.rectangle((181, 96, 197, 101), fill=c("d8d0ba"))
    draw.line((183, 98, 195, 97), fill=c("5f4e40"))
    draw.line((183, 100, 192, 100), fill=c("5f4e40"))

    for x in (34, 276):
        draw.line((x, 22, x, 56), fill=c("302b25"), width=2)
        draw.arc((x - 6, 52, x + 6, 64), 0, 180, fill=c("b08848"), width=1)

    for _ in range(260):
        x = int((math.sin(_ * 12.989) * 43758.5453) % 310)
        y = int((math.sin(_ * 78.233) * 24634.6345) % 120)
        if 5 < y < 105:
            draw.point((x, y), fill=(116, 91, 75, 120))

    high = low.resize((1240, 480), Image.Resampling.NEAREST).convert("RGB")
    out = ROOT / "assets" / "scenes" / "video_sewer_archive.png"
    high.save(out)
    print(out)


def draw_kliemannsland_road_room() -> None:
    scale = 4
    low = Image.new("RGBA", (310, 120), c("74a0bd"))
    draw = ImageDraw.Draw(low)

    for y in range(48):
        sky = (
            102 + y // 4,
            145 + y // 6,
            176 + y // 8,
            255,
        )
        draw.line((0, y, 310, y), fill=sky)

    for x, y, w in [(18, 12, 44), (91, 9, 62), (202, 14, 50), (240, 8, 46)]:
        draw.ellipse((x, y, x + w, y + 13), fill=(214, 224, 221, 255))
        draw.ellipse((x + 15, y - 5, x + w + 18, y + 10), fill=(232, 238, 234, 255))

    draw.rectangle((0, 37, 310, 71), fill=c("4d6b42"))
    for x in range(0, 310, 11):
        height = 16 + ((x * 17) % 19)
        draw.polygon([(x, 70), (x + 7, 70), (x + 3, 70 - height)], fill=c("2f4f35"))

    draw.polygon([(0, 56), (78, 51), (120, 120), (0, 120)], fill=c("5b5046"))
    draw.polygon([(104, 53), (310, 65), (310, 120), (118, 120)], fill=c("514c46"))
    draw.polygon([(80, 54), (104, 53), (118, 120), (120, 120)], fill=c("d5d1bd"))
    draw.polygon([(104, 53), (113, 54), (154, 120), (143, 120)], fill=c("2c2c2b"))
    draw.polygon([(172, 63), (180, 63), (236, 120), (220, 120)], fill=c("302f2d"))

    for y in [73, 91, 108]:
        draw.line((5, y, 100, y - 8), fill=c("c9c0a0"), width=1)
    for x in range(15, 97, 13):
        draw.line((x, 58, x - 4, 89), fill=c("2a2926"), width=2)
    draw.line((0, 63, 95, 56), fill=c("dbd0a4"), width=2)

    draw.rectangle((0, 45, 20, 73), fill=c("8c4f39"))
    draw.polygon([(0, 45), (22, 37), (22, 45)], fill=c("b47445"))
    draw.rectangle((24, 48, 60, 68), fill=c("273432"))
    draw.rectangle((28, 50, 56, 63), fill=c("151d1d"))

    draw.rectangle((187, 52, 207, 64), fill=c("f1ede2"))
    draw.rectangle((190, 48, 209, 55), fill=c("b83428"))
    draw.rectangle((213, 50, 239, 66), fill=c("3f3c39"))
    draw.rectangle((217, 46, 236, 58), fill=c("c94834"))
    draw.circle((228, 67), 4, fill=c("171717"))
    draw.circle((198, 65), 4, fill=c("171717"))

    draw.rectangle((248, 43, 269, 64), fill=c("435854"))
    draw.rectangle((254, 31, 258, 44), fill=c("394642"))
    draw.rectangle((244, 29, 268, 34), fill=c("d2b34a"))
    draw.rectangle((285, 50, 293, 77), fill=c("e7e0c8"))
    draw.rectangle((282, 49, 296, 54), fill=c("c84636"))

    for x, y in [(128, 53), (140, 57), (151, 62), (164, 68)]:
        draw.rectangle((x, y, x + 7, y + 16), fill=c("b72823"))
        draw.rectangle((x, y + 3, x + 7, y + 5), fill=c("efe7ca"))
        draw.rectangle((x, y + 8, x + 7, y + 10), fill=c("efe7ca"))

    draw.rectangle((111, 83, 131, 99), fill=c("222c2e"))
    draw.rectangle((114, 86, 128, 94), fill=c("d4b34b"))
    draw.line((112, 100, 104, 116), fill=c("d4b34b"), width=2)
    draw.line((130, 100, 143, 116), fill=c("d4b34b"), width=2)

    draw.rectangle((148, 95, 170, 101), fill=c("d7d1bb"))
    draw.line((151, 98, 166, 97), fill=c("6c5848"), width=1)
    draw.rectangle((224, 82, 250, 91), fill=c("1e2524"))
    draw.rectangle((226, 84, 248, 89), fill=c("d0b050"))

    for _ in range(220):
        x = int((math.sin(_ * 13.77) * 51437.13) % 310)
        y = int((math.sin(_ * 41.21) * 13817.42) % 120)
        if y > 52:
            draw.point((x, y), fill=(120, 112, 99, 110))

    high = low.resize((1240, 480), Image.Resampling.NEAREST).convert("RGB")
    out = ROOT / "assets" / "scenes" / "video_kliemannsland_road.png"
    high.save(out)
    print(out)


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
    draw_sewer_room()
    draw_kliemannsland_road_room()
    extend_inventory_icons()


if __name__ == "__main__":
    main()
