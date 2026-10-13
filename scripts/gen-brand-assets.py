#!/usr/bin/env python3
"""Regenerate Oshabe tray icons and in-app logo SVGs from the master symbol geometry.

Usage: python3 scripts/gen-brand-assets.py   (needs: pip install cairosvg)
"""
import pathlib

import cairosvg

ROOT = pathlib.Path(__file__).resolve().parent.parent
RES = ROOT / "src-tauri" / "resources"
BRAND = ROOT / "docs" / "brand"

INDIGO, AMBER, NIGHT = "#4338CA", "#FFB224", "#1B1830"
BUBBLE = (
    '<circle cx="490" cy="505" r="255"/>'
    '<path d="M312 660 L238 805 Q232 818 247 815 L408 740 Z"/>'
)
SPARK = '<circle cx="742" cy="227" r="36"/>'


def bars(kind):
    if kind == "idle":
        return [(324, 449, 120), (460, 355, 300), (596, 429, 160)]
    if kind == "recording":
        return [(324, 355, 300), (460, 355, 300), (596, 355, 300)]
    return []


def cutouts(kind):
    """SVG shapes (black = cut out of the bubble) for each tray state."""
    if kind in ("idle", "recording"):
        return "".join(
            f'<rect x="{x}" y="{y}" width="60" height="{h}" rx="30"/>' for x, y, h in bars(kind)
        )
    if kind == "transcribing":
        return "".join(f'<circle cx="{x}" cy="505" r="32"/>' for x in (354, 490, 626))
    if kind == "warning":
        return (
            '<rect x="460" y="375" width="60" height="150" rx="30"/>'
            '<circle cx="490" cy="610" r="34"/>'
        )
    raise ValueError(kind)


def svg(inner, bg=None):
    rect = f'<rect x="170" y="170" width="680" height="680" fill="{bg}"/>' if bg else ""
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="170 170 680 680">'
        f"{rect}{inner}</svg>"
    )


def mono(kind, color):
    """Returns (shape_svg, cutout_svg); composed in png_mono (cairosvg lacks reliable masks)."""
    spark = SPARK if kind in ("idle", "recording") else ""
    return (
        svg(f'<g fill="{color}">{BUBBLE}{spark}</g>'),
        svg(f'<g fill="#000">{cutouts(kind)}</g>'),
    )


def colored(kind):
    if kind == "idle":
        return svg(f'<g fill="{INDIGO}">{BUBBLE}</g><g fill="{AMBER}">{cutouts(kind)}{SPARK}</g>')
    if kind == "recording":
        return svg(f'<g fill="{AMBER}">{BUBBLE}{SPARK}</g><g fill="{NIGHT}">{cutouts(kind)}</g>')
    if kind == "transcribing":
        return svg(f'<g fill="{INDIGO}">{BUBBLE}</g><g fill="{AMBER}">{cutouts(kind)}</g>')
    return svg(f'<g fill="{AMBER}">{BUBBLE}</g><g fill="{NIGHT}">{cutouts(kind)}</g>')


def png_mono(pair, path, size=64):
    import io

    from PIL import Image, ImageChops

    def render(t):
        return Image.open(
            io.BytesIO(cairosvg.svg2png(bytestring=t.encode(), output_width=size * 4, output_height=size * 4))
        ).convert("RGBA")

    shape, cut = render(pair[0]), render(pair[1])
    alpha = ImageChops.multiply(shape.getchannel("A"), ImageChops.invert(cut.getchannel("A")))
    shape.putalpha(alpha)
    shape.resize((size, size), Image.LANCZOS).save(path)


def png(svg_text, path, size=64):
    cairosvg.svg2png(bytestring=svg_text.encode(), write_to=str(path), output_width=size, output_height=size)


def main():
    # Dark OS theme uses light icons, light OS theme uses dark icons (see tray.rs).
    for kind, stem in (
        ("idle", "tray_idle"),
        ("recording", "tray_recording"),
        ("transcribing", "tray_transcribing"),
        ("warning", "tray_idle_warning"),
    ):
        png_mono(mono(kind, "#FFFFFF"), RES / f"{stem}.png")
        png_mono(mono(kind, "#000000"), RES / f"{stem}_dark.png")
    # Colored set (Linux default).
    png(colored("idle"), RES / "handy.png")
    png(colored("recording"), RES / "recording.png")
    png(colored("transcribing"), RES / "transcribing.png")
    png(colored("warning"), RES / "handy_warning.png")


if __name__ == "__main__":
    main()
