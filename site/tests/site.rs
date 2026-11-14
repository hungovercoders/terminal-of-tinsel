//! Builds the site into a scratch directory and checks what came out.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use terminal_of_tinsel::creatures::Vault;
use tinsel_site::{BACKGROUND, BASE_URL, Config, build, contrast, readable};

/// Builds the site once per test into its own directory.
fn built(name: &str) -> PathBuf {
    let out = std::env::temp_dir().join(format!("tinsel-site-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&out);
    let cfg = Config {
        out: out.clone(),
        base_url: BASE_URL.to_string(),
    };
    build(&cfg).expect("the site builds");
    out
}

fn html_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(html_files(&path));
        } else if path.extension().is_some_and(|e| e == "html") {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Every value of an attribute in a page, such as each `href`.
fn attributes(html: &str, name: &str) -> Vec<String> {
    let needle = format!(" {name}=\"");
    html.match_indices(&needle)
        .map(|(at, _)| {
            let rest = &html[at + needle.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
        .collect()
}

#[test]
fn there_is_a_page_for_everything() {
    let out = built("pages");
    let vault = Vault::load();
    for path in [
        "index.html",
        "creatures/index.html",
        "advent.html",
        "privacy.html",
        "404.html",
        "sitemap.xml",
        "robots.txt",
        "llms.txt",
        "_headers",
        "static/style.css",
        "static/site.js",
        "static/themes.css",
        "static/fonts/fonts.css",
    ] {
        assert!(out.join(path).is_file(), "{path} is missing");
    }
    for c in &vault.creatures {
        let page = fs::read_to_string(out.join(format!("creatures/{}.html", c.id))).unwrap();
        assert!(page.contains(&tinsel_site::esc(&c.name)), "{}", c.id);
        assert!(page.contains("<h2>The legend</h2>"), "{}", c.id);
    }
    assert_eq!(html_files(&out).len(), vault.creatures.len() + 5);
}

#[test]
fn every_internal_link_leads_somewhere() {
    let out = built("links");
    for file in html_files(&out) {
        let html = fs::read_to_string(&file).unwrap();
        let mut targets = attributes(&html, "href");
        targets.extend(attributes(&html, "src"));
        for target in targets {
            if target.starts_with("https://") || target.starts_with("data:") {
                continue;
            }
            let (path, anchor) = match target.split_once('#') {
                Some((path, anchor)) => (path, Some(anchor)),
                None => (target.as_str(), None),
            };
            let resolved = match path {
                "" => file.clone(),
                // The 404 page is served at any address, so it links from the root.
                rooted if rooted.starts_with('/') => out.join(rooted.trim_start_matches('/')),
                relative => file.parent().unwrap().join(relative),
            };
            assert!(
                resolved.is_file(),
                "{}: {target} leads nowhere",
                file.display()
            );
            if let Some(anchor) = anchor {
                let page = fs::read_to_string(&resolved).unwrap();
                assert!(
                    page.contains(&format!("id=\"{anchor}\"")),
                    "{}: nothing is called #{anchor}",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn no_page_needs_more_than_the_content_security_policy_allows() {
    let out = built("csp");
    let headers = fs::read_to_string(out.join("_headers")).unwrap();
    assert!(headers.contains("script-src 'self';"));
    assert!(headers.contains("style-src 'self';"));
    for file in html_files(&out) {
        let html = fs::read_to_string(&file).unwrap();
        let name = file.display();
        assert!(!html.contains(" style=\""), "{name}: an inline style");
        assert!(!html.contains("<style"), "{name}: a style element");
        assert!(!html.contains(" onclick="), "{name}: an inline handler");
        for (at, _) in html.match_indices("<script") {
            let tag = &html[at..at + html[at..].find('>').unwrap()];
            assert!(tag.contains(" src=\""), "{name}: an inline script");
        }
        for target in attributes(&html, "src") {
            assert!(!target.starts_with("https://"), "{name}: loads {target}");
        }
    }
}

#[test]
fn every_page_has_what_a_search_engine_and_a_screen_reader_need() {
    let out = built("meta");
    let mut titles = BTreeSet::new();
    for file in html_files(&out) {
        let html = fs::read_to_string(&file).unwrap();
        let name = file.display();
        assert!(html.contains("<html lang=\"en-GB\">"), "{name}");
        assert_eq!(html.matches("<h1").count(), 1, "{name}: one h1");
        assert!(
            html.contains("<meta name=\"description\" content=\""),
            "{name}"
        );
        assert!(
            html.contains("<link rel=\"canonical\" href=\"https://"),
            "{name}"
        );
        let start = html.find("<title>").unwrap() + 7;
        let title = &html[start..html.find("</title>").unwrap()];
        assert!(
            titles.insert(title.to_string()),
            "{name}: {title} is used twice"
        );
    }
}

#[test]
fn creature_colours_can_be_read_on_the_background() {
    let vault = Vault::load();
    for c in &vault.creatures {
        for hex in [&c.theme.primary, &c.theme.accent] {
            let shown = readable(hex);
            assert!(
                contrast(&shown, BACKGROUND) >= 4.5,
                "{}: {hex} became {shown}",
                c.id
            );
        }
    }
    // A colour that is already readable is left alone.
    assert_eq!(readable("#F1FAEE"), "#F1FAEE");
}

#[test]
fn the_sitemap_and_llms_txt_list_every_creature() {
    let out = built("lists");
    let vault = Vault::load();
    let sitemap = fs::read_to_string(out.join("sitemap.xml")).unwrap();
    let llms = fs::read_to_string(out.join("llms.txt")).unwrap();
    for c in &vault.creatures {
        let address = format!("{BASE_URL}creatures/{}.html", c.id);
        assert!(sitemap.contains(&address), "sitemap: {}", c.id);
        assert!(llms.contains(&address), "llms.txt: {}", c.id);
    }
    assert!(!sitemap.contains("404.html"));
    assert!(sitemap.contains(&format!("<loc>{BASE_URL}</loc>")));
}

#[test]
fn the_advent_calendar_has_a_door_for_every_night() {
    let out = built("advent");
    let html = fs::read_to_string(out.join("advent.html")).unwrap();
    for night in 1..=24 {
        assert!(
            html.contains(&format!("data-night=\"{night}\"")),
            "door {night}"
        );
    }
}

#[test]
fn text_from_the_data_is_escaped() {
    assert_eq!(
        tinsel_site::esc("<b>\"Tom & Jerry's\"</b>"),
        "&lt;b&gt;&quot;Tom &amp; Jerry&#39;s&quot;&lt;/b&gt;"
    );
}
