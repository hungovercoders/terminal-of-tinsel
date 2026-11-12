//! The explorer, driven key by key and drawn into ratatui's `TestBackend`.

use jiff::civil::date;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::model::{Model, Options, Screen, tabs_for};
use super::view;
use crate::creatures::Vault;
use crate::rng::Rng;

fn options() -> Options {
    Options {
        show_all: false,
        start_id: None,
        no_intro: false,
        reveal: false,
        colour: true,
        today: date(2026, 11, 12),
        rng: Rng::new(25),
    }
}

fn press(m: &mut Model, code: KeyCode) {
    m.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn type_text(m: &mut Model, text: &str) {
    for ch in text.chars() {
        press(m, KeyCode::Char(ch));
    }
}

/// Draws the model at a size and returns the screen as text.
fn screen(m: &mut Model, width: u16, height: u16) -> String {
    crate::tui::snapshot(m, width, height)
}

fn on_page<'a>(vault: &'a Vault, id: &str) -> Model<'a> {
    Model::new(
        vault,
        Options {
            start_id: Some(id.to_string()),
            ..options()
        },
    )
}

#[test]
fn every_page_draws_at_every_size() {
    let vault = Vault::load();
    for (width, height) in [(1, 1), (20, 6), (40, 12), (80, 24), (100, 30), (160, 50)] {
        for c in &vault.creatures {
            let mut m = on_page(&vault, &c.id);
            for tab in 0..tabs_for(c).len() {
                press(&mut m, KeyCode::Char(char::from(b'1' + tab as u8)));
                press(&mut m, KeyCode::Char('r'));
                press(&mut m, KeyCode::End);
                let text = screen(&mut m, width, height);
                if width >= 80 && height >= 24 {
                    assert!(
                        text.contains(&c.name.to_uppercase()),
                        "{} at {width}x{height}, tab {tab}:\n{text}",
                        c.id
                    );
                }
            }
        }
        let mut m = Model::new(
            &vault,
            Options {
                show_all: true,
                no_intro: true,
                ..options()
            },
        );
        for _ in 0..vault.creatures.len() {
            press(&mut m, KeyCode::Down);
            screen(&mut m, width, height);
        }
        press(&mut m, KeyCode::Char('?'));
        press(&mut m, KeyCode::Char('/'));
        type_text(&mut m, "the");
        screen(&mut m, width, height);
    }
}

#[test]
fn the_intro_draws_every_frame_at_every_size() {
    let vault = Vault::load();
    for (width, height) in [(1, 1), (20, 6), (50, 16), (80, 24), (160, 50)] {
        let mut m = Model::new(&vault, options());
        for _ in 0..120 {
            screen(&mut m, width, height);
            m.on_tick();
        }
        assert!(m.intro_done());
        if width >= 80 {
            let text = screen(&mut m, width, height);
            assert!(text.contains("Auntie Aerial"), "{text}");
            assert!(text.contains("Press any key"), "{text}");
        }
    }
}

#[test]
fn the_first_key_finishes_the_intro_and_the_next_enters() {
    let vault = Vault::load();
    let mut m = Model::new(&vault, options());
    assert_eq!(m.screen, Screen::Intro);
    assert!(m.tick_every().is_some());
    press(&mut m, KeyCode::Enter);
    assert_eq!(m.screen, Screen::Intro);
    assert!(m.intro_done());
    assert!(m.seen().is_empty());
    press(&mut m, KeyCode::Enter);
    assert_eq!(m.screen, Screen::Detail);
    assert_eq!(m.seen(), ["krampus"]);
    // An ordinary page is still: nothing to animate, nothing to redraw.
    assert!(m.tick_every().is_none());
}

#[test]
fn the_intro_shows_tonights_notes_and_greets_christmas_eve() {
    let vault = Vault::load();
    let mut m = Model::new(
        &vault,
        Options {
            today: date(2026, 12, 24),
            ..options()
        },
    );
    press(&mut m, KeyCode::Enter);
    let text = screen(&mut m, 100, 30);
    assert!(text.contains("It's Christmas Eve"), "{text}");
    assert!(text.contains("13 Yule Lads are in town."), "{text}");
}

#[test]
fn q_quits_from_the_intro_with_a_sign_off() {
    let vault = Vault::load();
    let mut m = Model::new(&vault, options());
    assert!(m.sign_off().is_none());
    press(&mut m, KeyCode::Char('q'));
    assert!(m.quitting());
    assert!(m.sign_off().is_some());
}

#[test]
fn ctrl_c_quits_from_anywhere_even_a_search() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "tomte");
    press(&mut m, KeyCode::Char('/'));
    press(&mut m, KeyCode::Char('q'));
    assert!(!m.quitting(), "q is a letter while searching");
    m.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(m.quitting());
}

#[test]
fn naming_a_creature_skips_the_intro() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "mari-lwyd");
    assert_eq!(m.screen, Screen::Detail);
    assert!(screen(&mut m, 80, 24).contains("THE MARI LWYD"));
    assert_eq!(m.seen(), ["mari-lwyd"]);
}

#[test]
fn all_opens_on_the_gallery() {
    let vault = Vault::load();
    let mut m = Model::new(
        &vault,
        Options {
            show_all: true,
            no_intro: true,
            ..options()
        },
    );
    assert_eq!(m.screen, Screen::Gallery);
    let text = screen(&mut m, 80, 40);
    assert!(text.contains("14 guests tonight"), "{text}");
    assert!(text.contains("── Nordic Yule ──"), "{text}");
    assert!(text.contains("▶ 😈 Krampus"), "{text}");
}

#[test]
fn the_gallery_wraps_round_and_opens_the_selection() {
    let vault = Vault::load();
    let mut m = Model::new(
        &vault,
        Options {
            show_all: true,
            no_intro: true,
            ..options()
        },
    );
    press(&mut m, KeyCode::Up);
    assert_eq!(m.cursor, vault.creatures.len() - 1);
    press(&mut m, KeyCode::Char('j'));
    assert_eq!(m.cursor, 0);
    press(&mut m, KeyCode::Down);
    press(&mut m, KeyCode::Enter);
    assert_eq!(m.screen, Screen::Detail);
    assert_eq!(m.current().id, vault.creatures[1].id);
    press(&mut m, KeyCode::Char('g'));
    assert_eq!(m.screen, Screen::Gallery);
    assert_eq!(m.cursor, 1);
}

#[test]
fn the_selection_stays_on_screen_in_a_short_gallery() {
    let vault = Vault::load();
    let mut m = Model::new(
        &vault,
        Options {
            show_all: true,
            no_intro: true,
            ..options()
        },
    );
    for c in &vault.creatures {
        let text = screen(&mut m, 80, 10);
        assert!(
            text.contains(&format!("▶ {} {}", c.emoji, c.name)),
            "{text}"
        );
        press(&mut m, KeyCode::Down);
    }
}

#[test]
fn arrows_and_letters_move_between_creatures() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "krampus");
    press(&mut m, KeyCode::Char('l'));
    assert_eq!(m.index, 1);
    press(&mut m, KeyCode::Char('n'));
    press(&mut m, KeyCode::Right);
    assert_eq!(m.index, 3);
    press(&mut m, KeyCode::Char('h'));
    press(&mut m, KeyCode::Char('p'));
    press(&mut m, KeyCode::Left);
    assert_eq!(m.index, 0);
    press(&mut m, KeyCode::Left);
    assert_eq!(m.index, vault.creatures.len() - 1);
    assert_eq!(m.seen().len(), 5);
}

#[test]
fn tab_and_numbers_switch_sections() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "marley");
    let tabs = m.tabs().len();
    assert!(screen(&mut m, 80, 30).contains("Festive Facts"));
    press(&mut m, KeyCode::Tab);
    assert!(screen(&mut m, 80, 30).contains("The Legend"));
    press(&mut m, KeyCode::BackTab);
    press(&mut m, KeyCode::BackTab);
    assert_eq!(m.tab, tabs - 1);
    press(&mut m, KeyCode::Char('1'));
    assert_eq!(m.tab, 0);
    press(&mut m, KeyCode::Char('9'));
    assert_eq!(m.tab, 0, "there is no ninth section");
}

#[test]
fn the_same_section_stays_open_from_one_creature_to_the_next() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "krampus");
    press(&mut m, KeyCode::Tab);
    let legend = m.current_tab();
    press(&mut m, KeyCode::Right);
    assert_eq!(m.current_tab(), legend);
}

#[test]
fn verdicts_stay_hidden_until_r() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "gryla");
    let myths = m
        .tabs()
        .iter()
        .position(|t| *t == super::model::Tab::Myths)
        .unwrap();
    press(&mut m, KeyCode::Char(char::from(b'1' + myths as u8)));
    let hidden = screen(&mut m, 100, 50);
    assert!(hidden.contains("Grýla is the mother of the Yule Lads."));
    assert!(
        !hidden.contains("TRUE") && !hidden.contains("FALSE"),
        "{hidden}"
    );
    press(&mut m, KeyCode::Char('r'));
    let shown = screen(&mut m, 100, 50);
    assert!(
        shown.contains("✔ TRUE") && shown.contains("✘ FALSE"),
        "{shown}"
    );
    // Moving on hides them again for the next creature.
    press(&mut m, KeyCode::Right);
    assert!(!m.revealed);
}

#[test]
fn search_finds_lore_and_opens_the_creature() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "krampus");
    press(&mut m, KeyCode::Char('/'));
    assert!(screen(&mut m, 80, 24).contains("Search names"));
    type_text(&mut m, "porridgx");
    assert!(screen(&mut m, 80, 24).contains("Nobody at the door"));
    press(&mut m, KeyCode::Backspace);
    type_text(&mut m, "e");
    let text = screen(&mut m, 80, 24);
    assert!(text.contains("/ porridge▌"), "{text}");
    assert!(text.contains("The Tomte"), "{text}");
    press(&mut m, KeyCode::Enter);
    assert!(!m.searching);
    assert_eq!(m.current().id, "tomte");
    // Escape leaves a search without going anywhere.
    press(&mut m, KeyCode::Char('/'));
    type_text(&mut m, "cat");
    press(&mut m, KeyCode::Esc);
    assert_eq!(m.current().id, "tomte");
    assert_eq!(m.screen, Screen::Detail);
}

#[test]
fn the_portrait_sits_beside_the_text_on_wide_screens() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "tomte");
    let art_line = vault.get("tomte").unwrap().ascii.lines().nth(8).unwrap();
    let wide = screen(&mut m, 120, 40);
    let row = wide
        .lines()
        .find(|l| l.contains(art_line))
        .expect("portrait");
    assert!(row.contains('│'), "the portrait is boxed: {row}");
    assert!(
        wide.lines()
            .any(|l| l.contains("╭") && l.contains("Festive Facts")),
        "the text starts beside the top of the box:\n{wide}"
    );
    // On a narrow screen it goes under the facts instead.
    let narrow = screen(&mut m, 80, 60);
    let facts = narrow.lines().position(|l| l.contains("Festive Facts"));
    let art = narrow.lines().position(|l| l.contains(art_line));
    assert!(facts < art, "{narrow}");
}

#[test]
fn scrolling_stops_at_the_ends() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "krampus");
    screen(&mut m, 60, 16);
    press(&mut m, KeyCode::Up);
    assert_eq!(m.scroll, 0);
    press(&mut m, KeyCode::End);
    let end = m.scroll;
    assert!(end > 0);
    assert!(screen(&mut m, 60, 16).contains("▲ more above"));
    press(&mut m, KeyCode::PageDown);
    assert_eq!(m.scroll, end);
    press(&mut m, KeyCode::Home);
    assert!(screen(&mut m, 60, 16).contains("▼ more below"));
    // A taller terminal needs less scrolling, and the offset follows.
    press(&mut m, KeyCode::End);
    screen(&mut m, 60, 200);
    assert_eq!(m.scroll, 0);
}

#[test]
fn help_lists_every_key() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "krampus");
    assert!(!screen(&mut m, 100, 30).contains("Quit: q or ctrl+c"));
    press(&mut m, KeyCode::Char('?'));
    let text = screen(&mut m, 100, 30);
    assert!(text.contains("Quit: q or ctrl+c"), "{text}");
    assert!(text.contains("Sections: tab/shift+tab or 1-7"), "{text}");
}

#[test]
fn dickens_is_an_engraving_with_drifting_snow() {
    let vault = Vault::load();
    let mut m = on_page(&vault, "marley");
    assert!(m.engraving());
    assert!(m.tick_every().is_some(), "the snow needs a tick");
    let before = screen(&mut m, 120, 40);
    assert!(before.contains("ENGRAVER'S INK · 1843"), "{before}");
    for _ in 0..3 {
        m.on_tick();
    }
    assert_ne!(before, screen(&mut m, 120, 40), "the snow should move");
    let mut plain = on_page(&vault, "krampus");
    assert!(!plain.engraving());
    assert!(!screen(&mut plain, 120, 40).contains("ENGRAVER'S INK"));
}

#[test]
fn no_colour_means_no_colour() {
    use ratatui::style::Color;
    let vault = Vault::load();
    let mut m = Model::new(
        &vault,
        Options {
            start_id: Some("marley".into()),
            colour: false,
            ..options()
        },
    );
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    m.resize(120, 40);
    terminal.draw(|frame| view::draw(&m, frame)).unwrap();
    for cell in terminal.backend().buffer().content() {
        assert_eq!((cell.fg, cell.bg), (Color::Reset, Color::Reset));
    }
}

/// `DUMP=1 cargo test dump_screens -- --nocapture` prints sample screens.
#[test]
fn dump_screens() {
    if std::env::var_os("DUMP").is_none() {
        return;
    }
    let vault = Vault::load();
    let mut intro = Model::new(&vault, options());
    for frame in [5, 14, 19, 40, 400] {
        while intro.frame < frame {
            intro.on_tick();
        }
        println!(
            "── intro, frame {frame} ──\n{}\n",
            screen(&mut intro, 100, 28)
        );
    }
    let mut gallery = Model::new(
        &vault,
        Options {
            show_all: true,
            no_intro: true,
            ..options()
        },
    );
    println!("── gallery ──\n{}\n", screen(&mut gallery, 120, 40));
    for id in ["krampus", "marley"] {
        let mut page = on_page(&vault, id);
        println!("── {id}, wide ──\n{}\n", screen(&mut page, 120, 36));
        println!("── {id}, narrow ──\n{}\n", screen(&mut page, 70, 24));
    }
}
