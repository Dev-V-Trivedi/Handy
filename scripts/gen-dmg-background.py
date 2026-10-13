#!/usr/bin/env python3
"""Render scripts/dmg-background.html to src-tauri/dmg/background.png (needs playwright + Chromium)."""
import pathlib

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parent.parent
with sync_playwright() as p:
    b = p.chromium.launch(executable_path="/opt/pw-browsers/chromium") if pathlib.Path("/opt/pw-browsers/chromium").exists() else p.chromium.launch()
    pg = b.new_page(viewport={"width": 660, "height": 400})
    pg.goto((ROOT / "scripts" / "dmg-background.html").as_uri())
    pg.wait_for_timeout(500)
    pg.screenshot(path=str(ROOT / "src-tauri" / "dmg" / "background.png"))
    b.close()
