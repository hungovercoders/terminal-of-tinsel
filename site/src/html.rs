//! The shell every page shares, and escaping.

use terminal_of_tinsel::host;
use terminal_of_tinsel::rng::Rng;

use crate::{NAME, Page, REPO, theme_class};

/// Text made safe to put in HTML, in an element or a quoted attribute.
pub fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// A page's absolute address. The home page is the site's root.
pub fn address(base: &str, path: &str) -> String {
    match path {
        "index.html" => base.to_string(),
        _ => format!("{base}{path}"),
    }
}

const FAVICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3Ctext y='.9em' font-size='90'%3E%F0%9F%8E%84%3C/text%3E%3C/svg%3E";

/// The whole document for a page.
pub fn shell(page: &Page, base: &str) -> String {
    let root = page.root();
    let title = match page.path.as_str() {
        "index.html" => format!("{NAME}: Channel 25's Midwinter Special, live in your terminal"),
        _ => format!("{} · {NAME}", page.title),
    };
    let url = address(base, &page.path);
    let mut classes = format!("section-{}", page.section);
    if let Some(id) = &page.theme {
        classes.push_str(&format!(" themed {}", theme_class(id)));
    }
    if page.engraving {
        classes.push_str(" engraving");
    }
    let current = |section: &str| {
        if page.section == section {
            " aria-current=\"page\""
        } else {
            ""
        }
    };
    // The same sign-off on every build, so the pages don't change for no reason.
    let sign_off = host::sign_off(&mut Rng::new(25));
    format!(
        r##"<!doctype html>
<html lang="en-GB">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<meta name="description" content="{description}">
<meta name="theme-color" content="{background}">
<link rel="canonical" href="{url}">
<meta property="og:type" content="website">
<meta property="og:site_name" content="{NAME}">
<meta property="og:title" content="{og_title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{url}">
<meta name="twitter:card" content="summary">
<link rel="icon" href="{FAVICON}">
<link rel="preload" href="{root}static/fonts/ibm-plex-sans-latin-400-normal.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="{root}static/fonts/playfair-display-latin-700-normal.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{root}static/fonts/fonts.css">
<link rel="stylesheet" href="{root}static/style.css">
<link rel="stylesheet" href="{root}static/themes.css">
</head>
<body class="{classes}">
<div class="snow" aria-hidden="true"></div>
<a class="skip" href="#main">Skip to content</a>
<header class="topbar">
  <a class="brand" href="{root}index.html"><span class="onair"><span class="dot"></span>ON AIR</span><span class="brand-name">{NAME}</span></a>
  <nav class="topnav" aria-label="Site">
    <a href="{root}index.html"{home}>Tonight</a>
    <a href="{root}creatures/index.html"{creatures}>The Roster</a>
    <a href="{root}advent.html"{advent}>Advent</a>
    <a href="{REPO}" rel="noopener">GitHub</a>
  </nav>
</header>
<main id="main">
{body}
</main>
<footer class="signoff">
  <p class="host-line">📺 <em>"{sign_off}"</em></p>
  <p class="small">{NAME} is free software, dedicated to the public domain under CC0. <a href="{REPO}">Source on GitHub</a> · <a href="{REPO}/releases/latest">Releases</a> · <a href="{REPO}/blob/main/CONTRIBUTING.md">Contribute a creature</a> · <a href="{root}privacy.html">Privacy</a><br>No cookies, no tracking scripts, no per-request logs.<br>{host} and {channel} are fictional. The creatures are folklore, and the quotations are from books long out of copyright.</p>
</footer>
<script src="{root}static/site.js" defer></script>
</body>
</html>
"##,
        title = esc(&title),
        og_title = esc(&page.title),
        description = esc(&page.description),
        background = crate::BACKGROUND,
        home = current("home"),
        creatures = current("creatures"),
        advent = current("advent"),
        body = page.body,
        sign_off = esc(sign_off),
        host = host::NAME,
        channel = host::CHANNEL,
    )
}
