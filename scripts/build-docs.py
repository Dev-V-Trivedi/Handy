#!/usr/bin/env python3
"""Build the docs page from docs/site-docs.md.

Writes site/docs/index.html, site/docs.md and site/llms-full.txt.
Run from the repo root: python3 scripts/build-docs.py
Needs: pip install markdown
"""
import html, json, re, sys
from pathlib import Path
import markdown

ROOT = Path(__file__).resolve().parent.parent
SRC = (ROOT / "docs" / "site-docs.md").read_text(encoding="utf-8")
SITE = "https://oshabe.hikariwebworks.studio"

md = markdown.Markdown(extensions=["fenced_code", "tables", "toc"], extension_configs={"toc": {"permalink": False}})
body = md.convert(SRC)
# Drop the H1 from the body, the page header shows it.
body = re.sub(r"<h1[^>]*>.*?</h1>\s*", "", body, count=1, flags=re.S)
intro_end = body.find("<h2")
intro, rest = body[:intro_end], body[intro_end:]

# Sidebar from h2 and h3.
toc = []
for m in re.finditer(r'<h([23]) id="([^"]+)">(.*?)</h\1>', rest, re.S):
    toc.append((int(m.group(1)), m.group(2), re.sub("<[^>]+>", "", m.group(3))))
nav = ""
for lvl, i, t in toc:
    nav += f'<a class="l{lvl}" href="#{i}">{html.escape(t)}</a>\n'

# Per-section copy buttons on h2.
def add_copy(m):
    return f'<h2 id="{m.group(1)}"><span>{m.group(2)}</span><button class="cp" type="button" data-sec="{m.group(1)}" aria-label="Copy this section for an LLM">Copy</button></h2>'
rest = re.sub(r'<h2 id="([^"]+)">(.*?)</h2>', add_copy, rest, flags=re.S)

faq_like = [
    ("How do I start meeting notes?", "Press the meeting shortcut (Ctrl+Alt+M on Windows and Linux, Option+Shift+M on macOS), use the tray menu, or click Start on the Meetings page."),
    ("Where does Oshabe store my data?", "In a local data folder: %APPDATA%\\com.hikariwebworks.oshabe on Windows, ~/Library/Application Support/com.hikariwebworks.oshabe on macOS and ~/.local/share/com.hikariwebworks.oshabe on Linux."),
]
ld = {
    "@context": "https://schema.org",
    "@graph": [
        {
            "@type": "TechArticle",
            "@id": f"{SITE}/docs/#article",
            "headline": "Oshabe documentation",
            "description": "How to install and use Oshabe: dictation, meeting notes, cleanup models, troubleshooting, data locations and command-line flags.",
            "url": f"{SITE}/docs/",
            "inLanguage": "en",
            "dateModified": "2026-10-15",
            "author": {"@type": "Organization", "name": "Hikari Webworks", "url": "https://hikariwebworks.studio"},
            "publisher": {"@type": "Organization", "name": "Hikari Webworks", "url": "https://hikariwebworks.studio"},
            "about": {"@id": f"{SITE}/#app"},
        },
        {
            "@type": "BreadcrumbList",
            "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Oshabe", "item": f"{SITE}/"},
                {"@type": "ListItem", "position": 2, "name": "Documentation", "item": f"{SITE}/docs/"},
            ],
        },
        {
            "@type": "FAQPage",
            "mainEntity": [
                {"@type": "Question", "name": q, "acceptedAnswer": {"@type": "Answer", "text": a}} for q, a in faq_like
            ],
        },
    ],
}

nav_links = '<a href="/#features">Features</a><a href="/#download">Download</a><a href="/docs/" aria-current="page">Docs</a>'
tpl = (ROOT / "scripts" / "docs-template.html").read_text(encoding="utf-8")
page = (
    tpl.replace("{{NAV}}", nav)
    .replace("{{INTRO}}", intro)
    .replace("{{BODY}}", rest)
    .replace("{{LD}}", json.dumps(ld, indent=2))
    .replace("{{MD}}", SRC.replace("</script", "<\\/script"))
    .replace("{{SITE}}", SITE)
)
out = ROOT / "site" / "docs" / "index.html"
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(page, encoding="utf-8")
(ROOT / "site" / "docs.md").write_text(SRC, encoding="utf-8")

overview = (ROOT / "site" / "llms.txt").read_text(encoding="utf-8")
(ROOT / "site" / "llms-full.txt").write_text(overview.rstrip() + "\n\n---\n\n" + SRC, encoding="utf-8")
print("built", out, len(page), "bytes")
