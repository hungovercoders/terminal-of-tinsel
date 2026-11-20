# Changing the website

The website is a static site generated from the repository by `site/`, a second crate in the Cargo
workspace. It reads the creatures, the advent calendar, the host's lines and the command list from the
program's own library, so a new creature or command appears on the site without touching it.

```bash
task site          # cargo run -p tinsel-site -- --out dist
task site:serve    # build, then serve dist/ on http://localhost:8025
cargo test -p tinsel-site
```

`dist/` is ignored by git. `--base-url` sets the address the canonical links and the sitemap use.

## Files

```
site/src/lib.rs      build(): writes every page, the static files, themes.css, sitemap.xml, robots.txt, llms.txt
site/src/html.rs     The shell every page shares (head, navigation, footer) and esc()
site/src/pages.rs    A function per page: home, roster, creature, advent, privacy, not_found
site/src/course.rs   Renders course/*.md into night-school/, rewriting links between lessons and into the repository
site/src/colour.rs   Lightens a creature's colours until they can be read on the background
site/static/         style.css, site.js and the fonts, copied as they are
site/tests/site.rs   Builds the site into a scratch directory and checks it
public/_headers      Cloudflare's response headers, the Content-Security-Policy among them
wrangler.jsonc       Cloudflare's configuration: serve dist/ as static assets
```

## Rules

- **Escape everything from the data.** Pages are built with `format!`, which escapes nothing. Every
  string that comes from a pack, the host or clap goes through `esc()`. Raw HTML literals are
  `r##"..."##`, because `href="#main"` would end an `r#"..."#` string.
- **No inline styles or scripts.** The Content-Security-Policy allows styles and scripts from the site
  alone. A creature's colours are a class in the generated `static/themes.css` (`.c-<id>` sets
  `--primary` and `--accent`), and stat bars are `.bar-0` to `.bar-10`. The tests fail a page with a
  `style` attribute, a `<style>` element or a script without a `src`.
- **Nothing from another site.** No fonts, scripts or images are loaded from elsewhere, and the privacy
  page says so. If that changes, change `public/_headers` and the privacy page in the same commit.
- **The pages work without JavaScript.** `site.js` only adds the copy buttons and opens tonight's
  advent door. Myths and advent doors are `<details>` elements.
- **Links are relative**, with `.html` names, so the site works from a folder as well as a server.
  `Page::root()` gives the way back to the top. The 404 page alone links from `/`, because it is served
  at any address.
- **Lessons are Markdown.** Raw HTML in a lesson is shown as text, not passed through, and a table with
  aligned columns would need an inline style, so lessons don't align columns.
- **Fonts** are self-hosted from the `@fontsource` packages, each with its OFL text beside it. A new
  font needs its licence file and an entry in `static/fonts/fonts.css`.
- The house rules still hold: British English, no em dashes, and no copyrighted characters or quotes.

## Adding a page

Write a function in `site/src/pages.rs` that returns a `Page`, add it to `pages()` in `site/src/lib.rs`,
link to it from the navigation in `html.rs` if it belongs there, and add its path to
`there_is_a_page_for_everything` in the tests. The sitemap and llms.txt pick it up from `pages()`;
llms.txt lists the fixed pages by hand, so add a line there too.

## Looking at it

The tests check structure, not looks. After a change to the styles, run `task site:serve` and look at
the home page, a creature from the Dickens pack (drawn as an engraving), a creature with a long name
and the advent calendar, at a phone's width and a desktop's.

## Deploying

Not set up yet. [website-plan.md](website-plan.md) lists what is left: a Cloudflare project, a domain
and the slopstopper `ui` profile.
