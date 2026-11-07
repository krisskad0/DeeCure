# The website

[Docs](../README.md) › [Packaging](README.md)

DeeCure's landing page lives at **https://krisskad0.github.io/DeeCure/**. It shows what DeeCure does, who it is for, screenshots, privacy, and an FAQ, and people can install or download DeeCure from it.

## Where it comes from

| Part | Where |
| --- | --- |
| The page | `site/` in `main`: `index.html`, `style.css`, `app.js`, `404.html`, `robots.txt`, `sitemap.xml`, and `assets/` (icons, the share image, the font) |
| Screenshots | Copied from `docs/07-user-guide/images/` when the site is published, so new screenshots reach the site by themselves |
| Publishing | `.github/workflows/pages.yml`: on a push to `main` that changes `site/` or the screenshots (or by hand), it copies them onto the `gh-pages` branch. GitHub Pages serves that branch. |
| APT repository | `apt/` on the same branch, written only by the release job. The page workflow never touches `apt/` or `.reprepro/`, and both workflows share the `gh-pages` concurrency group, so they never push at the same time. |

## How it works

- **Download links** come from GitHub's release list in the browser. The page picks the highest version, as `install.sh` and the in-app updater do (GitHub once listed alpha.9 before alpha.10). Without JavaScript or when GitHub is not reachable, the "All releases on GitHub" link stays.
- **No tracking:** no analytics, no cookies, no web fonts from other sites (Ubuntu Sans is served from the site, with its licence), and no third-party scripts. The only outside request is GitHub's release list.
- **Light and dark** follow the system. The layout works from phone width up, and motion stops when the system asks for less motion.

## Search and sharing

- A clear title and description, a canonical URL, and `robots.txt` with the `sitemap.xml`.
- Open Graph and Twitter cards with `assets/og-image.png` (1200 × 630). See "Share image" below.

## Share image

The poster people see when the site or the repo is shared. It says "Every Linux system hides problems. DeeCure finds them." beside the dark Overview screenshot. Its source is [scripts/og-image.html](../../scripts/og-image.html), which uses the site's font, icon, and screenshot.

1. Copy the screenshots into `site/assets/screens/` first (the folder is ignored by git).
2. Render the site image, from the repo folder:
   `google-chrome --headless=new --hide-scrollbars --force-device-scale-factor=1 --allow-file-access-from-files --virtual-time-budget=3000 --window-size=1200,630 --screenshot=site/assets/og-image.png "file://$PWD/scripts/og-image.html"`
3. Render the GitHub social preview the same way with `--window-size=1280,640` and any other output file. Upload it in the repo's **Settings › General › Social preview** (GitHub has no API for it).
4. Look at both images before you commit.
- Structured data: `SoftwareApplication` (free, Ubuntu 24.04 and 26.04, features, screenshots) and `FAQPage` (the same questions as the page).
- Every image has a full description in `alt`, with its width and height.

## Change the page

1. Edit the files in `site/`. To preview, copy the screenshots into `site/assets/screens/` (ignored by git) and serve the folder as `/DeeCure/`, for example with `python3 -m http.server` from a folder that links `DeeCure` to `site/`.
2. Keep the FAQ on the page and in the `FAQPage` data the same.
3. Push to `main`. The Pages workflow publishes it in about a minute.
