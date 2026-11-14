//! Auntie Aerial's Night School on the website: the lessons in `course/`,
//! rendered from Markdown.

use std::fs;
use std::io;
use std::path::Path;

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

use crate::{Page, REPO};

/// The folder the course's pages are written to.
const FOLDER: &str = "night-school";

/// One Markdown file of the course, rendered.
pub struct Lesson {
    /// The page's name under the course folder, such as `night-03.html`.
    pub file: String,
    /// The lesson's heading, such as "Night 3 · Meet the Guests".
    pub title: String,
    pub html: String,
}

/// The course: its index and its lessons, in order.
pub struct Course {
    pub index: Lesson,
    pub lessons: Vec<Lesson>,
}

impl Course {
    /// Reads `README.md` and every `night-NN.md` from the course folder.
    pub fn load(dir: &Path) -> io::Result<Course> {
        let index = lesson(&dir.join("README.md"), "index.html")?;
        let mut names: Vec<String> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .filter(|name| name.starts_with("night-") && name.ends_with(".md"))
            .collect();
        names.sort();
        let lessons = names
            .iter()
            .map(|name| lesson(&dir.join(name), &name.replace(".md", ".html")))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Course { index, lessons })
    }

    /// The course's pages: the index, then a page per lesson.
    pub fn pages(&self) -> Vec<Page> {
        let page = |lesson: &Lesson, description: String| Page {
            path: format!("{FOLDER}/{}", lesson.file),
            title: lesson.title.clone(),
            description,
            section: "school",
            theme: None,
            engraving: false,
            body: format!(
                "<article class=\"prose lesson\">\n{}</article>\n",
                lesson.html
            ),
        };
        let mut pages = vec![page(
            &self.index,
            "A free course that teaches Rust by building Terminal of Tinsel from an empty folder, one lesson for each night of advent.".to_string(),
        )];
        pages.extend(self.lessons.iter().map(|lesson| {
            page(
                lesson,
                format!(
                    "{}: a lesson from Auntie Aerial's Night School, the course that teaches Rust by building Terminal of Tinsel.",
                    lesson.title
                ),
            )
        }));
        pages
    }

    /// The course's entries for llms.txt.
    pub fn llms_txt(&self, base: &str) -> String {
        let mut text = format!(
            "\n## Auntie Aerial's Night School\n\nA course that teaches Rust by building the program, one lesson for each night of advent.\n\n- [The programme]({base}{FOLDER}/index.html)\n"
        );
        for lesson in &self.lessons {
            text.push_str(&format!(
                "- [{}]({base}{FOLDER}/{})\n",
                lesson.title, lesson.file
            ));
        }
        text
    }
}

fn lesson(path: &Path, file: &str) -> io::Result<Lesson> {
    let markdown = fs::read_to_string(path)?;
    let title = markdown
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} has no '# ' heading for its title", path.display()),
            )
        })?;
    Ok(Lesson {
        file: file.to_string(),
        title: plain(title),
        html: render(&markdown),
    })
}

/// A heading with its Markdown and emoji decoration taken off, for a title.
fn plain(heading: &str) -> String {
    heading
        .chars()
        .filter(|ch| !matches!(ch, '*' | '`' | '_'))
        .collect::<String>()
        .trim_start_matches(|ch: char| !ch.is_alphanumeric())
        .trim()
        .to_string()
}

/// Markdown to HTML. Links between lessons become links between pages,
/// links into the repository go to GitHub, and raw HTML is shown as text,
/// so a lesson can't bring in a script or an inline style.
fn render(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let events = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::from(link(&dest_url)),
            title,
            id,
        }),
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

/// Where a link in a lesson leads on the website.
pub fn link(dest: &str) -> String {
    if dest.contains("://") || dest.starts_with('#') || dest.starts_with("mailto:") {
        return dest.to_string();
    }
    let (path, anchor) = match dest.split_once('#') {
        Some((path, anchor)) => (path, format!("#{anchor}")),
        None => (dest, String::new()),
    };
    if path == "README.md" {
        return format!("index.html{anchor}");
    }
    if path.starts_with("night-") && path.ends_with(".md") {
        return format!("{}{anchor}", path.replace(".md", ".html"));
    }
    // Anything else is a file in the repository, relative to course/.
    let in_repo = match path.strip_prefix("../") {
        Some(rest) => rest.to_string(),
        None => format!("course/{path}"),
    };
    let kind = if in_repo.ends_with('/') {
        "tree"
    } else {
        "blob"
    };
    format!("{REPO}/{kind}/main/{in_repo}{anchor}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_between_lessons_become_links_between_pages() {
        assert_eq!(link("night-02.md"), "night-02.html");
        assert_eq!(link("README.md"), "index.html");
        assert_eq!(link("night-07.md#tests"), "night-07.html#tests");
    }

    #[test]
    fn links_into_the_repository_go_to_github() {
        assert_eq!(
            link("../src/quiz.rs"),
            format!("{REPO}/blob/main/src/quiz.rs")
        );
        assert_eq!(link("../packs/"), format!("{REPO}/tree/main/packs/"));
        assert_eq!(
            link("https://www.rust-lang.org"),
            "https://www.rust-lang.org"
        );
        assert_eq!(link("#try-it"), "#try-it");
    }

    #[test]
    fn raw_html_in_a_lesson_is_shown_not_run() {
        let html = render("Hello <script>alert(1)</script> there\n\n<div style=\"x\">hi</div>\n");
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<div"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn a_title_loses_its_decoration() {
        assert_eq!(
            plain("📺 Auntie Aerial's Night School"),
            "Auntie Aerial's Night School"
        );
        assert_eq!(
            plain("Night 3 · Meet the Guests"),
            "Night 3 · Meet the Guests"
        );
    }
}
