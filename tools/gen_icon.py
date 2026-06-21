#!/usr/bin/env python3
"""Generate the ClipEdit master app icon (1024x1024 RGBA PNG) with no external
dependencies. Run `tauri icon src-tauri/app-icon.png` afterwards to produce all
platform sizes."""
import math
import struct
import zlib

W = H = 1024
buf = bytearray(W * H * 4)  # transparent


def clamp(x, a, b):
    return a if x < a else (b if x > b else x)


def over(x, y, r, g, b, a):
    """Alpha-composite (r,g,b,a) over the existing pixel."""
    i = (y * W + x) * 4
    dr, dg, db, da = buf[i], buf[i + 1], buf[i + 2], buf[i + 3]
    sa = a / 255.0
    da_ = da / 255.0
    outa = sa + da_ * (1 - sa)
    if outa <= 0:
        return
    buf[i] = int((r * sa + dr * da_ * (1 - sa)) / outa + 0.5)
    buf[i + 1] = int((g * sa + dg * da_ * (1 - sa)) / outa + 0.5)
    buf[i + 2] = int((b * sa + db * da_ * (1 - sa)) / outa + 0.5)
    buf[i + 3] = int(outa * 255 + 0.5)


def rrect_cov(px, py, cx, cy, hw, hh, r):
    """Anti-aliased coverage of a rounded rectangle at a pixel center."""
    dx = abs(px - cx) - (hw - r)
    dy = abs(py - cy) - (hh - r)
    outside = math.hypot(max(dx, 0.0), max(dy, 0.0)) - r
    inside = min(max(dx, dy), 0.0)
    return clamp(0.5 - (outside + inside), 0.0, 1.0)


def rrect(cx, cy, hw, hh, r, color, alpha=255):
    x0 = int(clamp(cx - hw - 2, 0, W - 1))
    x1 = int(clamp(cx + hw + 2, 0, W - 1))
    y0 = int(clamp(cy - hh - 2, 0, H - 1))
    y1 = int(clamp(cy + hh + 2, 0, H - 1))
    cr, cg, cb = color
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            cov = rrect_cov(x + 0.5, y + 0.5, cx, cy, hw, hh, r)
            if cov > 0:
                over(x, y, cr, cg, cb, alpha * cov)


def tile(cx, cy, hw, hh, r, top, bottom):
    """Rounded square with a vertical gradient."""
    x0 = int(clamp(cx - hw - 2, 0, W - 1))
    x1 = int(clamp(cx + hw + 2, 0, W - 1))
    y0 = int(clamp(cy - hh - 2, 0, H - 1))
    y1 = int(clamp(cy + hh + 2, 0, H - 1))
    for y in range(y0, y1 + 1):
        t = clamp((y - (cy - hh)) / (2 * hh), 0, 1)
        cr = top[0] + (bottom[0] - top[0]) * t
        cg = top[1] + (bottom[1] - top[1]) * t
        cb = top[2] + (bottom[2] - top[2]) * t
        for x in range(x0, x1 + 1):
            cov = rrect_cov(x + 0.5, y + 0.5, cx, cy, hw, hh, r)
            if cov > 0:
                over(x, y, cr, cg, cb, 255 * cov)


# Background tile (blue gradient)
tile(512, 512, 420, 420, 190, (0x5B, 0x9B, 0xFF), (0x3C, 0x78, 0xF0))
# Clipboard tab sticking out above the board
rrect(512, 322, 96, 58, 28, (0xD8, 0xE2, 0xF0))
# Clipboard board (white)
rrect(512, 552, 196, 232, 40, (0xFF, 0xFF, 0xFF))
# Content lines
line = (0xB9, 0xCC, 0xEA)
accent = (0x4F, 0x8C, 0xFF)
rrect(498, 470, 120, 17, 17, accent)
rrect(512, 540, 134, 17, 17, line)
rrect(512, 610, 134, 17, 17, line)
rrect(470, 680, 92, 17, 17, line)


def chunk(typ, data):
    return (
        struct.pack(">I", len(data))
        + typ
        + data
        + struct.pack(">I", zlib.crc32(typ + data) & 0xFFFFFFFF)
    )


raw = bytearray()
for y in range(H):
    raw.append(0)  # filter type 0
    raw += buf[y * W * 4 : (y + 1) * W * 4]

png = b"\x89PNG\r\n\x1a\n"
png += chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 6, 0, 0, 0))
png += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
png += chunk(b"IEND", b"")

out = "/Users/christianptc_/Projects/clipedit/src-tauri/app-icon.png"
with open(out, "wb") as f:
    f.write(png)
print("wrote", out, len(png), "bytes")
