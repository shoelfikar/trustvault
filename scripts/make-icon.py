#!/usr/bin/env python3
"""Render src-tauri/icon-source.png, the master image every app icon is generated from.

The mark is the one the design prototype uses in its own header ("TrustVault App.dc.html",
a 64x64 viewBox): a warm-graphite tile, a brass rounded-square outline, and a brass disc with
a padlock knocked out of it.

This script exists so the icon is reproducible from the repository rather than being a binary
nobody can regenerate. Without it, icon-source.png would be a dead end — committed because it
could not be recreated, which is the weakest reason to commit anything.

    python3 scripts/make-icon.py          # writes src-tauri/icon-source.png
    npm run icons                          # then fans it out into src-tauri/icons/

Requires Pillow. Colours are the tokens from design-system/password-manager/MASTER.md; if the
brand colour changes there, change it here too.
"""

from pathlib import Path

from PIL import Image, ImageDraw

VIEWBOX = 64  # the source artwork's coordinate space
EDGE = 1024  # final icon edge in pixels
SUPERSAMPLE = 4  # Pillow has no analytic anti-aliasing, so draw big and shrink

SCALE = EDGE * SUPERSAMPLE / VIEWBOX

BASE = (19, 18, 17, 255)  # --bg-base  #131211
BRASS = (192, 138, 46, 255)  # --accent   #C08A2E

REPO_ROOT = Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "src-tauri" / "icon-source.png"


def u(value: float) -> float:
    """Convert a viewBox coordinate to a supersampled pixel coordinate."""
    return value * SCALE


def box(x: float, y: float, w: float, h: float) -> list[float]:
    """A viewBox rectangle as the [x0, y0, x1, y1] Pillow expects."""
    return [u(x), u(y), u(x + w), u(y + h)]


def render() -> Image.Image:
    img = Image.new("RGBA", (EDGE * SUPERSAMPLE, EDGE * SUPERSAMPLE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Full-bleed tile. The corner radius follows the mark's own rx=13 over its 46-unit box.
    draw.rounded_rectangle(
        [0, 0, EDGE * SUPERSAMPLE - 1, EDGE * SUPERSAMPLE - 1],
        radius=u(VIEWBOX * 0.22),
        fill=BASE,
    )

    # Brass rounded-square outline: rect x=9 y=9 w=46 h=46 rx=13, stroke 5.
    draw.rounded_rectangle(box(9, 9, 46, 46), radius=u(13), outline=BRASS, width=int(u(5)))

    # The disc.
    draw.ellipse(box(32 - 13, 32 - 13, 26, 26), fill=BRASS)

    # The padlock, knocked out of the disc. Drawn in BASE rather than cleared to transparent:
    # clearing would punch a hole through the tile and show the desktop behind the icon.
    draw.rounded_rectangle(box(28.1, 31.5, 7.8, 8.4), radius=u(1.6), fill=BASE)

    stroke = u(2.5)
    draw.arc(box(32 - 4.5, 29.2 - 4.5, 9, 9), start=180, end=360, fill=BASE, width=int(stroke))
    draw.rectangle([u(27.5), u(29.2), u(27.5) + stroke, u(31.5)], fill=BASE)
    draw.rectangle([u(36.5) - stroke, u(29.2), u(36.5), u(31.5)], fill=BASE)

    return img.resize((EDGE, EDGE), Image.LANCZOS)


def main() -> None:
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    render().save(OUTPUT)
    print(f"wrote {OUTPUT.relative_to(REPO_ROOT)} ({EDGE}x{EDGE})")


if __name__ == "__main__":
    main()
