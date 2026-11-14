# The website (phase 8)

Terror's site is generated from the repository by `go run ./site` and published to Cloudflare by Workers
Builds on every push to `main`. Tinsel's follows the same shape. The generator is built; the site is not
deployed. [website.md](website.md) says how to change it.

| | Terror | Tinsel |
|-|--------|--------|
| [x] | `site/` Go program reading `internal/monsters` | `site/` is a second crate in a Cargo workspace (`tinsel-site`), reading the program's own library (`src/lib.rs`); cargo-dist still ships only the CLI |
| [x] | `html/template` files, one per page kind plus a base | plain Rust: `site/src/html.rs` is the shell and `site/src/pages.rs` a function per page, with no template dependency |
| [x] | pages: home, roster, one per monster, privacy, 404 | home, roster, one per creature with its portrait and colours, the advent calendar (a Tinsel extra), privacy, 404 |
| [x] | course pages rendered from `course/` | `site/src/course.rs` renders each lesson with pulldown-cmark into `night-school/`; no code highlighter, so nothing is loaded from another site |
| [x] | `sitemap.xml`, `robots.txt`, `llms.txt` generated | the same |
| [x] | no inline styles or scripts; CSP in `public/_headers`; per-monster colours as generated CSS classes | the same, and stricter: nothing is loaded from another site |
| [x] | self-hosted fonts with their licences | Playfair Display, IBM Plex Sans and IBM Plex Mono, with their OFL texts |
| [x] | README install, games and rituals copy repeated on the landing page | the same; the command table is generated from clap |
| [ ] | hero GIF and screenshots from `docs/assets`; a social card | waits for the demo recordings; the pages carry no `og:image` until then |
| [x] | `go test ./site` builds the site and checks every internal link | `cargo test -p tinsel-site`: links, the CSP, colour contrast, the sitemap and llms.txt |
| [ ] | `wrangler.jsonc`, Workers Builds deploys `main` | `wrangler.jsonc` is written but nothing is connected: it needs a Cloudflare project, a domain, and a check that Workers Builds can run `cargo` (if not, build in GitHub Actions and deploy with wrangler) |
| [ ] | slopstopper profile `ui`: smoke, e2e, accessibility, SEO, links, web vitals, DAST | switch the profile and set the production URL once the site is live |
| [x] | a doc on changing the website | [website.md](website.md) |

## Before it goes live

- **The address.** `BASE_URL` in `site/src/lib.rs` is `https://terminaloftinsel.com/`, by analogy with
  Terror's. It is a placeholder until a domain is agreed: the canonical links, the sitemap and llms.txt
  are built from it. Change the constant, or pass `--base-url`.
- **The custom domain** goes in `wrangler.jsonc` as a route.
- **The privacy page** says per-request logs are off and nothing is loaded from other sites. Both are
  true of the configuration in the repository; check them again against the Cloudflare project.
