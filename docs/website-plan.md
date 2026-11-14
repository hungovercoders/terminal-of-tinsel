# The website (phase 8)

Terror's site is generated from the repository by `go run ./site` and published to Cloudflare by Workers
Builds on every push to `main`. Tinsel's follows the same shape.

| | Terror | Tinsel |
|-|--------|--------|
| [ ] | `site/` Go program reading `internal/monsters` | a `site` workspace member reading the same pack data as the CLI, which means moving the CLI's modules behind a `src/lib.rs`; cargo-dist keeps shipping only the CLI |
| [ ] | `html/template` files, one per page kind plus a base | the same pages from a small template crate (to be vetted) or plain Rust |
| [ ] | pages: home, roster, one per monster, privacy, 404 | home, roster, one per creature with its portrait and colours, the advent calendar (a Tinsel extra), privacy, 404 |
| [ ] | course pages rendered from `course/` | only if the Night School stretch goal goes ahead |
| [ ] | `sitemap.xml`, `robots.txt`, `llms.txt` generated | the same |
| [ ] | no inline styles or scripts; CSP in `public/_headers`; per-monster colours as generated CSS classes | the same |
| [ ] | self-hosted fonts with their licences | the same, in a Christmas-card pairing |
| [ ] | README install, games and rituals copy repeated on the landing page | the same |
| [ ] | hero GIF and screenshots from `docs/assets`; a social card | the same, from the P7 recordings |
| [ ] | `go test ./site` builds the site and checks every internal link | a test that builds into a temp dir and checks every link |
| [ ] | `wrangler.jsonc`, Workers Builds deploys `main` | the same, once a Cloudflare project and domain are agreed |
| [ ] | slopstopper profile `ui`: smoke, e2e, accessibility, SEO, links, web vitals, DAST | switch the profile and set the production URL |
| [ ] | a doc on changing the website | the same |
