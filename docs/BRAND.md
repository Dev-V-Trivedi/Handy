# Oshabe brand plan

Status: plan only. Nothing here is applied to the app yet.

## Concept

**Oshabe** comes from Japanese _oshaberi_, "chatter". Hikari (light) is the studio behind it, so the brand is a voice that glows: a friendly speech bubble with a spark of light.

Tone: warm, quick, a little playful, never corporate. Local and private, not "AI magic".

## Logo system

| Part                     | Description                                                                                                                                       |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Symbol                   | Rounded speech bubble, three vertical rounded bars inside (sound wave, also reads as a mouth saying "o"), one small spark circle at the top right |
| Wordmark                 | lowercase `oshabe`, typeset in Fredoka SemiBold (set by us, not drawn by an image model, so spelling is always right)                             |
| Lockups                  | symbol above or beside wordmark, light and dark versions                                                                                          |
| Japanese line (optional) | おしゃべ in M PLUS Rounded 1c, used on marketing only                                                                                             |

Rules for the symbol: only circles, rounded rectangles and one small tail (easy to rebuild as clean SVG), two colours maximum, no gradients, no outlines, readable at 16 px.

## Assets to request from ChatGPT

Ask for these four first. Generate the master symbol first, pick one, then ask for the variants with "use this exact design".

| #   | File name               | What                             | Size      | Notes                                                                                                                          |
| --- | ----------------------- | -------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 1   | `oshabe-mark-light.png` | Symbol for light backgrounds     | 1024x1024 | Bubble indigo `#4338CA`, bars and spark amber `#FFB224`, transparent background                                                |
| 2   | `oshabe-mark-dark.png`  | Same symbol for dark backgrounds | 1024x1024 | Bubble amber `#FFB224`, bars night `#1B1830`, spark amber, transparent background                                              |
| 3   | `oshabe-mark-mono.png`  | Single colour silhouette         | 1024x1024 | Pure black, transparent background. Used for tray icons and favicon, so keep shapes chunky                                     |
| 4   | `oshabe-app-icon.png`   | App icon                         | 1024x1024 | Solid night `#1B1830` square, dark symbol centered at about 60% width. No rounded corners, border or shadow (the OS adds them) |

Optional, lower priority:

| File name            | What                                                                                                             |
| -------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `oshabe-mascot.png`  | Small friendly character for README and empty states (a chatty bird or the bubble with a face). Same two colours |
| `oshabe-hero-bg.png` | Soft abstract background for the README banner and social preview, 1536x1024, indigo and amber glow on night     |

If ChatGPT cannot output a transparent background, ask for a plain white background. We cut it out.

If it can, ask for the symbol as **SVG code** too. A symbol built from circles and rounded rectangles is simple enough that SVG code is cleaner than a traced image.

### Copy-paste prompts

**1. Master symbol (light)**

> Design a logo symbol, with no text, for a voice dictation app called Oshabe. The name comes from the Japanese word oshaberi, meaning chatter. Concept: a rounded, friendly speech bubble with three vertical rounded bars inside it of different heights, like a sound wave. A small solid circle, a spark of light, sits just outside the bubble at the top right. Flat vector style built only from circles, rounded rectangles and one small bubble tail. Exactly two colours: indigo #4338CA for the bubble, amber #FFB224 for the bars and the spark. No gradients, no shadows, no outlines, no texture, no text. Bold and simple, readable at 16 pixels. Centered with generous padding, square 1:1, transparent background.

**2. Dark variant**

> Use this exact design and proportions. Change only the colours: bubble amber #FFB224, the three bars dark navy #1B1830, spark amber #FFB224. Transparent background.

**3. Mono silhouette**

> Use this exact design. Make it a single flat black silhouette on a transparent background: bubble solid black, bars and spark as cut-outs so the transparency shows through. Keep every shape chunky enough to read at 16 pixels.

**4. App icon**

> Use this exact dark-colour symbol. Place it centered on a solid #1B1830 square background, symbol about 60% of the width. No rounded corners, no border, no shadow, no gradient. 1024x1024.

## What we generate ourselves from those four files

| Output                                                                         | Where it goes                                                                                           |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| All desktop, iOS, Android and Store icons (`icon.ico`, `icon.icns`, `icons/*`) | `bun run tauri icon oshabe-app-icon.png`                                                                |
| 8 tray icons, 64x64 (idle, recording, transcribing, warning, light and dark)   | `src-tauri/resources/tray_*.png`, plus colored `handy.png`, `recording.png`, `transcribing.png` renamed |
| In-app logo (sidebar, onboarding)                                              | replaces `OshabeTextLogo.tsx` with symbol plus real wordmark                                            |
| Horizontal and stacked lockups, light and dark                                 | `docs/brand/`                                                                                           |
| GitHub social preview (1280x640), README banner                                | `docs/brand/`                                                                                           |
| Installer sidebar and header images (optional)                                 | `src-tauri/nsis`                                                                                        |
| Favicon                                                                        | `index.html`                                                                                            |

## Color palette

| Name            | Hex                             | Role                                            |
| --------------- | ------------------------------- | ----------------------------------------------- |
| Koe Indigo      | `#4338CA`                       | Primary actions, logo on light                  |
| Koe Indigo Deep | `#3730A3`                       | Hover and pressed                               |
| Hikari Amber    | `#FFB224`                       | Spark, recording glow, logo on dark, highlights |
| Night           | `#1B1830`                       | Dark background, app icon                       |
| Washi           | `#FBF8F3`                       | Light background, text on dark                  |
| Sumi            | `#17141F`                       | Text on light                                   |
| Lavender        | `#A59BFF`                       | Links and focus rings on dark                   |
| Mid gray        | `#808080`                       | Unchanged from today: borders, secondary text   |
| Error           | `#DC2626` light, `#F87171` dark | Unchanged                                       |
| Warning         | `#D97706` light, `#FBBF24` dark | Unchanged                                       |

Contrast checked (WCAG ratio):

| Pair                    | Ratio | Verdict                                              |
| ----------------------- | ----- | ---------------------------------------------------- |
| White on Indigo         | 7.9   | pass, button text                                    |
| Indigo on Washi         | 7.5   | pass, links                                          |
| Sumi on Washi           | 17.1  | pass, body text                                      |
| Amber on Night          | 9.6   | pass                                                 |
| Washi on Night          | 16.3  | pass, body text                                      |
| Lavender on Night       | 7.2   | pass                                                 |
| Sumi on Amber           | 10.1  | pass                                                 |
| Indigo on Night         | 2.2   | fail, so never use indigo text on dark; use Lavender |
| Warning orange on Washi | 3.0   | marginal, use only with an icon, not as text         |

### Mapping to the existing tokens (`src/styles/theme.css`)

| Token                            | Light     | Dark      |
| -------------------------------- | --------- | --------- |
| `--color-text`                   | `#17141F` | `#FBF8F3` |
| `--color-background`             | `#FBF8F3` | `#1B1830` |
| `--color-logo-primary`           | `#4338CA` | `#FFB224` |
| `--color-logo-stroke`            | `#1B1830` | `#FFE2A8` |
| `--color-background-ui` (shared) | `#4338CA` | `#4338CA` |

Today's pink (`#faa2ca`, `#da5893`) is replaced everywhere through these tokens, so it is a one-file change. Check hover and focus states after the swap, because many of them are `logo-primary` tints.

## Fonts

| Use                                        | Font                       | Why                                                            |
| ------------------------------------------ | -------------------------- | -------------------------------------------------------------- |
| Wordmark and marketing headings            | **Fredoka** SemiBold       | Rounded and friendly, matches the bubble shapes                |
| App UI and body                            | **Inter** (variable)       | Clear at small sizes, wide Latin, Cyrillic and Greek coverage  |
| Japanese accents                           | **M PLUS Rounded 1c**      | Rounded kana that match Fredoka                                |
| Other scripts (Hindi, Telugu, Arabic, CJK) | system fallback, Noto Sans | Shipping every script would add many MB; the OS provides these |

All three are SIL Open Font License, so bundling and redistribution are fine. Bundle them with `@fontsource` packages (self-hosted). Do **not** load Google Fonts from the CDN: the app is local-first and should not make network calls for styling.

Stack: `font-family: "Inter Variable", system-ui, -apple-system, "Segoe UI", "Noto Sans", sans-serif;`

## Order of work

1. You generate the four images (above) and send them over.
2. We vectorize the symbol, build the wordmark and all lockups.
3. Run `tauri icon`, generate tray icons, replace the in-app logo and favicon.
4. Swap the palette tokens in `theme.css`, check light and dark in the app.
5. Bundle fonts, set the font stack.
6. Social preview and README banner.

Name clearance is still open (see `docs/REBRAND-NOTES.md`). Do not print merch or buy a domain on the strength of this plan alone.
