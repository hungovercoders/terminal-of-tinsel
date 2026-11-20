//! The body of each page, built from the creature data.

use clap::CommandFactory;
use terminal_of_tinsel::cli::Cli;
use terminal_of_tinsel::creatures::{Creature, Film, Pack, Vault};
use terminal_of_tinsel::host;
use terminal_of_tinsel::rng::Rng;

use crate::{Page, REPO, esc, theme_class};

/// Whether a pack is drawn as an engraving.
fn is_engraving(pack: Option<&Pack>) -> bool {
    pack.is_some_and(|p| p.style.as_deref() == Some("engraving"))
}

/// A block of terminal text with a button to copy it.
fn code(text: &str) -> String {
    format!(
        "<div class=\"code\"><pre><code>{}</code></pre><button class=\"copy\" type=\"button\" hidden>Copy</button></div>",
        esc(text)
    )
}

/// A creature's card, as the roster and the home page show it.
fn card(c: &Creature, root: &str, engraving: bool) -> String {
    format!(
        "<li class=\"card {theme}{style}\"><a href=\"{root}creatures/{id}.html\"><span class=\"card-emoji\" aria-hidden=\"true\">{emoji}</span><span class=\"card-name\">{name}</span><span class=\"card-origin\">{origin}</span><span class=\"card-text\">{text}</span></a></li>",
        theme = theme_class(&c.id),
        style = if engraving { " engraving" } else { "" },
        id = c.id,
        emoji = esc(&c.emoji),
        name = esc(&c.name),
        origin = esc(&c.origin),
        text = esc(&c.description),
    )
}

fn list(items: &[String]) -> String {
    let items: String = items
        .iter()
        .map(|item| format!("<li>{}</li>", esc(item)))
        .collect();
    format!("<ul>{items}</ul>")
}

const INSTALL: [(&str, &str); 4] = [
    (
        "With Homebrew, on macOS or Linux",
        "brew install hungovercoders/tap/terminal-of-tinsel",
    ),
    (
        "With the install script, on macOS or Linux",
        "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hungovercoders/terminal-of-tinsel/releases/latest/download/terminal-of-tinsel-installer.sh | sh",
    ),
    (
        "On Windows, in PowerShell",
        "powershell -ExecutionPolicy Bypass -c \"irm https://github.com/hungovercoders/terminal-of-tinsel/releases/latest/download/terminal-of-tinsel-installer.ps1 | iex\"",
    ),
    (
        "With Rust 1.88 or newer, from source",
        "cargo install --locked --git https://github.com/hungovercoders/terminal-of-tinsel",
    ),
];

const FIRST_NIGHT: &str = "terminal-of-tinsel                   # Auntie Aerial says hello
terminal-of-tinsel creature          # tune in to the Midwinter Special
terminal-of-tinsel list              # meet tonight's guests
terminal-of-tinsel creature krampus  # read a creature's page
terminal-of-tinsel quiz              # pull a cracker: the Christmas Cracker Quiz
terminal-of-tinsel random --daily    # the Fact of the Night
terminal-of-tinsel tonight           # tonight's Christmas double bill
terminal-of-tinsel countdown         # sleeps until Christmas
terminal-of-tinsel advent            # open the advent calendar";

/// What's on: a title, the command that starts it and what it is.
const PROGRAMMES: [(&str, &str, &str); 8] = [
    (
        "The Midwinter Special",
        "creature",
        "Channel 25 tunes in through static and snow, then a gallery of every guest and a page each: the legend, an original portrait, Myth or Fact?, powers, quotations and their night at the pictures. Arrow keys or h and l to change guest, / to search.",
    ),
    (
        "The Christmas Cracker Quiz",
        "quiz",
        "Multiple-choice questions made from the creature data, with every answer explained, and a rank at the end from Stocking Filler to Star of the Tree.",
    ),
    (
        "Guess the Guest",
        "guess",
        "A portrait behind a frosted window. Clear more frost for more clues, at the cost of points.",
    ),
    (
        "The Snowball Showdown",
        "mash",
        "Three rounds between any two creatures, decided by their stats and a die. The Yule Cat against Krampus has been settled several ways.",
    ),
    (
        "The Stocking",
        "stocking",
        "Three right answers about a creature put it in your stocking. There are badges to earn and records to beat, saved in a file on your own machine.",
    ),
    (
        "Tonight's double bill",
        "tonight",
        "A cinema ticket for the night's two features, the same all day and different tomorrow.",
    ),
    (
        "The countdown",
        "countdown",
        "The sleeps until Christmas in big digits, with tonight's moon and the next special night of the season.",
    ),
    (
        "Packs",
        "packs",
        "The roster comes in packs, and you can write your own: a folder of JSON files in your config directory joins every command.",
    ),
];

pub fn home(vault: &Vault) -> Page {
    let greeting = host::greeting(&mut Rng::new(25));
    let install: String = INSTALL
        .iter()
        .map(|(how, command)| format!("<h3>{}</h3>{}", esc(how), code(command)))
        .collect();
    let programmes: String = PROGRAMMES
        .iter()
        .map(|(title, command, text)| {
            format!(
                "<li class=\"programme\"><h3>{}</h3><p>{}</p><p class=\"run\"><code>terminal-of-tinsel {}</code></p></li>",
                esc(title),
                esc(text),
                esc(command)
            )
        })
        .collect();
    // One guest from each pack, to show what the roster holds.
    let guests: String = vault
        .by_pack()
        .iter()
        .flat_map(|(pack, creatures)| {
            let engraving = is_engraving(Some(pack));
            creatures
                .iter()
                .take(2)
                .map(move |c| card(c, "", engraving))
        })
        .collect();
    let commands: String = Cli::command()
        .get_subcommands()
        .filter(|sub| sub.get_name() != "help")
        .map(|sub| {
            format!(
                "<tr><th scope=\"row\"><code>{}</code></th><td>{}</td></tr>",
                esc(sub.get_name()),
                esc(&sub.get_about().map(ToString::to_string).unwrap_or_default())
            )
        })
        .collect();
    let body = format!(
        r##"<section class="hero">
  <p class="kicker">{channel} · The Midwinter Special</p>
  <h1>Terminal of Tinsel</h1>
  <p class="tagline">The creatures of Christmas folklore, live in your terminal.</p>
  <p class="host-line">📺 <em>"{greeting}"</em></p>
  <p class="cta-row"><a class="btn" href="#install">Unwrap it</a> <a class="btn ghost" href="creatures/index.html">Meet the {count} guests</a></p>
</section>

<figure class="screen">
  <img src="assets/hero.gif" width="1148" height="748" alt="A recording of the program: Channel 25 tunes in through falling snow, then the explorer pages through Krampus, the Yule Cat and the Tomte, searches for porridge and signs off.">
</figure>

<section class="prose">
  <p>St Nicholas has grim companions. Iceland has a cat that eats anyone without new clothes, and thirteen lads who come down from the mountains one by one. In Wales a horse's skull knocks at the door and wants a rhyme. Terminal of Tinsel tells their real stories, with what the folklore says and what it doesn't, presented by your host, {host}.</p>
  <p>It is the Christmas sister of <a href="https://github.com/hungovercoders/terminal-of-terror">Terminal of Terror</a>. Terror is written in Go and Tinsel in Rust, to see how the two compare.</p>
</section>

<section id="install" class="prose">
  <h2>Unwrap it</h2>
  {install}
</section>

<section class="prose">
  <h2>Your first night</h2>
  {first_night}
  <p>Add <code>terminal-of-tinsel random --daily</code> to your shell's startup file and every new terminal opens with the Fact of the Night. From 1 to 24 December it is about the guest behind that night's <a href="advent.html">advent door</a>.</p>
</section>

<section>
  <h2>What's on</h2>
  <ul class="programmes">{programmes}</ul>
</section>

<section>
  <h2>Tonight's guests</h2>
  <ul class="cards">{guests}</ul>
  <p class="cta-row"><a class="btn ghost" href="creatures/index.html">See the whole roster</a></p>
</section>

<section class="prose">
  <h2>Night School</h2>
  <p>Want to know how the picture gets made? <a href="night-school/index.html">Auntie Aerial's Night School</a> is a free course that teaches Rust by building this program from an empty folder, one lesson for each night of advent.</p>
</section>

<section class="prose">
  <h2>Every command</h2>
  <table class="commands">
    <thead><tr><th scope="col">Command</th><th scope="col">What it does</th></tr></thead>
    <tbody>{commands}</tbody>
  </table>
  <p>Every command takes <code>--json</code> for scripts, and <code>--no-color</code> for plain output with no colour, no emoji and ASCII borders.</p>
</section>
"##,
        channel = host::CHANNEL,
        host = host::NAME,
        greeting = esc(greeting),
        count = vault.creatures.len(),
        first_night = code(FIRST_NIGHT),
    );
    Page {
        path: "index.html".into(),
        title: "Terminal of Tinsel".into(),
        description: "A command-line program about the creatures of Christmas folklore: Krampus, the Yule Cat, the Mari Lwyd and the spirits of A Christmas Carol, with games, an advent calendar and a countdown to Christmas.".into(),
        section: "home",
        theme: None,
        engraving: false,
        body,
    }
}

pub fn roster(vault: &Vault) -> Page {
    let packs: String = vault
        .by_pack()
        .iter()
        .map(|(pack, creatures)| {
            let engraving = is_engraving(Some(pack));
            let cards: String = creatures.iter().map(|c| card(c, "../", engraving)).collect();
            format!(
                "<section><h2>{}</h2><p class=\"pack-text\">{}</p><ul class=\"cards\">{cards}</ul></section>",
                esc(&pack.name),
                esc(&pack.description)
            )
        })
        .collect();
    let body = format!(
        "<section class=\"prose\"><p class=\"kicker\">{} · Tonight's guests</p><h1>The Roster</h1><p>{} creatures in {} packs. Each has a page here, and a fuller one in your terminal.</p></section>{packs}",
        host::CHANNEL,
        vault.creatures.len(),
        vault.packs.len(),
    );
    Page {
        path: "creatures/index.html".into(),
        title: "The Roster".into(),
        description: format!(
            "All {} creatures of Christmas folklore in Terminal of Tinsel, by pack: St Nicholas's companions, Nordic Yule, Midwinter Europe and A Christmas Carol.",
            vault.creatures.len()
        ),
        section: "creatures",
        theme: None,
        engraving: false,
        body,
    }
}

fn film(f: &Film) -> String {
    let mut lines = vec![format!(
        "<p><strong>{}</strong> ({}){}</p>",
        esc(&f.title),
        f.year,
        if f.silent { ", a silent film" } else { "" }
    )];
    let mut credits = vec![
        ("Directed by", Some(&f.director)),
        ("Starring", Some(&f.star)),
        ("Studio", f.studio.as_ref()),
        ("Make-up", f.makeup.as_ref()),
    ];
    credits.retain(|(_, who)| who.is_some_and(|w| !w.is_empty()));
    let credits: String = credits
        .iter()
        .map(|(role, who)| {
            format!(
                "<dt>{role}</dt><dd>{}</dd>",
                esc(who.map(String::as_str).unwrap_or_default())
            )
        })
        .collect();
    lines.push(format!("<dl class=\"credits\">{credits}</dl>"));
    if let Some(note) = &f.note {
        lines.push(format!("<p>{}</p>", esc(note)));
    }
    lines.concat()
}

/// The page for the creature at this place in the roster.
pub fn creature(vault: &Vault, index: usize) -> Page {
    let c = &vault.creatures[index];
    let pack = vault.pack_of(c);
    let count = vault.creatures.len();
    let previous = &vault.creatures[(index + count - 1) % count];
    let next = &vault.creatures[(index + 1) % count];

    let mut sections = Vec::new();
    if !c.ascii.is_empty() {
        sections.push(format!(
            "<figure class=\"portrait\"><pre role=\"img\" aria-label=\"An ASCII art portrait of {}\">{}</pre></figure>",
            esc(&c.name),
            esc(&c.ascii)
        ));
    }
    sections.push(format!(
        "<p class=\"host-line\">📺 <em>\"{}\"</em></p>",
        esc(&host::intro(c))
    ));
    sections.push(format!(
        "<section><h2>The legend</h2>{}</section>",
        c.legend
            .split("\n\n")
            .map(|p| format!("<p>{}</p>", esc(p)))
            .collect::<String>()
    ));
    let stats: String = c
        .stats
        .labelled()
        .iter()
        .map(|(label, value)| {
            format!(
                "<dt>{label}</dt><dd><span class=\"bar bar-{v}\" aria-hidden=\"true\"></span><span class=\"score\">{value}<span class=\"visually-hidden\"> out of 10</span></span></dd>",
                v = (*value).min(10)
            )
        })
        .collect();
    sections.push(format!(
        "<section><h2>Form guide</h2><dl class=\"stats\">{stats}</dl></section>"
    ));
    if !c.powers.is_empty() {
        sections.push(format!(
            "<section><h2>Powers</h2>{}</section>",
            list(&c.powers)
        ));
    }
    if !c.weaknesses.is_empty() {
        sections.push(format!(
            "<section><h2>Weaknesses</h2>{}</section>",
            list(&c.weaknesses)
        ));
    }
    if !c.myths.is_empty() {
        let myths: String = c
            .myths
            .iter()
            .map(|m| {
                format!(
                    "<details class=\"myth\"><summary>{}</summary><p><strong>{}</strong> {}</p></details>",
                    esc(&m.claim),
                    if m.is_true { "Fact." } else { "Myth." },
                    esc(&m.explanation)
                )
            })
            .collect();
        sections.push(format!(
            "<section><h2>Myth or Fact?</h2><p class=\"small\">Decide, then open each one for the verdict.</p>{myths}</section>"
        ));
    }
    if !c.quotes.is_empty() {
        let quotes: String = c
            .quotes
            .iter()
            .map(|q| {
                format!(
                    "<figure class=\"quote\"><blockquote><p>{}</p></blockquote><figcaption>{}, <cite>{}</cite></figcaption></figure>",
                    esc(&q.text),
                    esc(&q.speaker),
                    esc(&q.source)
                )
            })
            .collect();
        sections.push(format!(
            "<section><h2>In their own words</h2>{quotes}</section>"
        ));
    }
    if let Some(f) = &c.film {
        sections.push(format!(
            "<section><h2>At the pictures</h2>{}</section>",
            film(f)
        ));
    }
    if !c.legacy.is_empty() {
        sections.push(format!(
            "<section><h2>Legacy</h2>{}</section>",
            list(&c.legacy)
        ));
    }
    if !c.facts.is_empty() {
        sections.push(format!(
            "<section><h2>Facts for the cracker</h2>{}</section>",
            list(&c.facts)
        ));
    }

    let aliases = if c.aliases.is_empty() {
        String::new()
    } else {
        format!(
            "<p class=\"aliases\">Also known as {}</p>",
            esc(&c.aliases.join(", "))
        )
    };
    let body = format!(
        r##"<article class="creature">
<header>
  <p class="kicker"><a href="index.html">The Roster</a> · {pack}</p>
  <h1><span aria-hidden="true">{emoji}</span> {name}</h1>
  {aliases}
  <p class="lede">{description}</p>
  <dl class="facts-line"><dt>From</dt><dd>{origin}</dd><dt>First appeared</dt><dd>{first}</dd></dl>
</header>
{sections}
<section>
  <h2>In your terminal</h2>
  {command}
</section>
<nav class="pager" aria-label="Creatures">
  <a rel="prev" href="{prev_id}.html">← {prev_name}</a>
  <a rel="next" href="{next_id}.html">{next_name} →</a>
</nav>
</article>
"##,
        pack = esc(pack.map(|p| p.name.as_str()).unwrap_or_default()),
        emoji = esc(&c.emoji),
        name = esc(&c.name),
        description = esc(&c.description),
        origin = esc(&c.origin),
        first = esc(&c.first_appearance()),
        sections = sections.join("\n"),
        command = code(&format!("terminal-of-tinsel creature {}", c.id)),
        prev_id = previous.id,
        prev_name = esc(&previous.name),
        next_id = next.id,
        next_name = esc(&next.name),
    );
    Page {
        path: format!("creatures/{}.html", c.id),
        title: c.name.clone(),
        description: c.description.clone(),
        section: "creatures",
        theme: Some(c.id.clone()),
        engraving: is_engraving(pack),
        body,
    }
}

pub fn advent(vault: &Vault) -> Page {
    let doors: String = vault
        .advent
        .iter()
        .filter_map(|night| Some((night, vault.get(&night.id)?)))
        .map(|(night, c)| {
            format!(
                "<li><details class=\"door {theme}\" data-night=\"{n}\"><summary><span class=\"door-number\">{n}</span><span class=\"visually-hidden\"> December</span></summary><p class=\"door-guest\"><span aria-hidden=\"true\">{emoji}</span> <a href=\"creatures/{id}.html\">{name}</a></p><p>{note}</p></details></li>",
                theme = theme_class(&c.id),
                n = night.night,
                emoji = esc(&c.emoji),
                id = c.id,
                name = esc(&c.name),
                note = esc(&night.note),
            )
        })
        .collect();
    let body = format!(
        r##"<section class="prose">
  <p class="kicker">{channel} · 1 to 24 December</p>
  <h1>The Advent Calendar</h1>
  <p>One guest behind each door. Open them here in any order you like; in your terminal the calendar keeps the days honestly, and a door stays shut until its night.</p>
  {command}
</section>
<ol class="doors">{doors}</ol>
"##,
        channel = host::CHANNEL,
        command = code("terminal-of-tinsel advent"),
    );
    Page {
        path: "advent.html".into(),
        title: "The Advent Calendar".into(),
        description: "Terminal of Tinsel's advent calendar: a different creature of Christmas folklore behind each door, from 1 to 24 December.".into(),
        section: "advent",
        theme: None,
        engraving: false,
        body,
    }
}

pub fn privacy() -> Page {
    let body = format!(
        r##"<article class="prose">
  <p class="kicker">{channel} · The small print</p>
  <h1>Privacy</h1>
  <p class="host-line">📺 <em>"I can see a great deal from the top of the tree. You are not part of it."</em></p>

  <h2>What this site collects</h2>
  <p>Nothing of its own. There are no accounts, no forms, no cookies and no analytics or tracking scripts. The site is a set of static pages whose only script is its own <code>site.js</code>, which adds the copy buttons and opens tonight's advent door. Its <a href="{REPO}/blob/main/public/_headers">Content-Security-Policy</a> allows scripts and styles from this site alone and none inline.</p>

  <h2>Hosting</h2>
  <p>The pages are served by <a href="https://www.cloudflare.com/">Cloudflare</a>. Like any host, Cloudflare processes your IP address and browser details to deliver each page, and it keeps aggregate traffic statistics (how many requests, from which countries, with which status codes). Per-request logging is switched off for this site, so no record of your individual visit is kept for the maintainers to read. Cloudflare's handling of that data is described in its <a href="https://www.cloudflare.com/privacypolicy/">privacy policy</a>.</p>

  <h2>Other services the pages talk to</h2>
  <p>None. Links go to GitHub, which has its <a href="https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement">own privacy statement</a>, but nothing is loaded from it. Fonts are served from this site, not from a font service.</p>

  <h2>The program itself</h2>
  <p><code>terminal-of-tinsel</code> makes no network requests. Your quiz scores, the creatures in your stocking and your badges live in one JSON file in your own config directory (<code>terminal-of-tinsel stocking</code> prints where), and <code>stocking reset</code> deletes it.</p>

  <h2>Questions</h2>
  <p>Open an issue on <a href="{REPO}/issues">GitHub</a>. This page was last checked against the site's configuration on 14 November 2026.</p>

  <p class="cta-row"><a class="btn ghost" href="index.html">Back to {channel}</a></p>
</article>
"##,
        channel = host::CHANNEL,
    );
    Page {
        path: "privacy.html".into(),
        title: "Privacy".into(),
        description:
            "What the Terminal of Tinsel website and program collect about you, which is nothing."
                .into(),
        section: "privacy",
        theme: None,
        engraving: false,
        body,
    }
}

pub fn not_found() -> Page {
    // Served for any missing address, so its links are from the site's root.
    let body = format!(
        r##"<article class="prose">
  <p class="kicker">{channel} · Lost in the snow</p>
  <h1>Nothing behind this door</h1>
  <p class="host-line">📺 <em>"We seem to have lost the picture. It'll be the tinsel on the aerial again."</em></p>
  <p class="cta-row"><a class="btn" href="/index.html">Back to {channel}</a> <a class="btn ghost" href="/creatures/index.html">The Roster</a></p>
</article>
"##,
        channel = host::CHANNEL,
    );
    Page {
        path: "404.html".into(),
        title: "Nothing behind this door".into(),
        description: "This page isn't on tonight's schedule.".into(),
        section: "lost",
        theme: None,
        engraving: false,
        body,
    }
}
