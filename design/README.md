# Design

UI sketches for mosaico, kept in the repo so the history of a design sits next to the code that implements it.
Sketches are drawn in FigCraft and built by scripts, so they can be regenerated after a
change to a theme or a layout rule.

Each feature has its own folder:

| Folder | Feature | Plan |
| --- | --- | --- |
| [`settings`](settings) | Settings app and tray icon | [Phase 37](../.plans/phase-37.md) |

A feature folder holds:

| File | Purpose |
| --- | --- |
| `*.figcraft` | The FigCraft document. Open it to inspect or edit the sketches by hand. |
| `all.js` | Rebuilds every window, then runs `export.js`. |
| `build.js` | Draws one settings window (title bar, sidebar, page, footer), replacing a window with the same name. |
| `pages.js` | The contents of each page, used by `build.js`. |
| `tray.js` | Draws the tray icon, tooltip, menu and notification sketch. |
| `ui.js` | Shared widgets: rows, sections, dropdowns, toggles, sliders, tabs and so on. |
| `theme.js` | Theme flavors, the UI color roles taken from them, and icons. |
| `export.js` | Renders windows to `png/`. With no arguments it renders all of them and saves the document. |
| `fgsession.js` | Talks to the running FigCraft app over its control port. |
| `png/` | Rendered windows at 2x, for reviews, issues and docs. |

## Look and feel

The settings app takes its colors from the active mosaico theme flavor (for example Catppuccin Mocha or Latte).
Everything else follows the ArtCraft apps: Inter, 13 px body text, 5 px widget radius, a title bar, a sidebar with
search and navigation, and settings grouped in bordered cards.

## Regenerating

Start FigCraft in control mode from the repo root, so it can read `assets/` and write here:

```powershell
& 'C:\Program Files\FigCraft\figcraft.exe' --control 7879 `
    --automation-read-root $PWD --automation-write-root "$PWD\design"
```

Open `design/settings/settings.figcraft`, then rebuild every window, save and export:

```powershell
node design/settings/all.js
```

To work on one window, rebuild it at its place in the grid (Mocha at x 0, Latte at x 1060, one row of 780 per
page in the order of `all.js`) and render only the windows whose names match:

```powershell
node design/settings/build.js Bar mocha 0 4680
node design/settings/export.js "Bar / Mocha"
```

Sketches are drawn in Catppuccin Mocha and Latte only. The app itself supports every theme.

`FG_PORT` overrides the control port and `FIGCRAFT_CLI` the path to `figcraft-cli.exe`.
