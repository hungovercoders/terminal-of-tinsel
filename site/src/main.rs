//! `cargo run -p tinsel-site -- --out dist` writes the website into `dist`.

use std::path::PathBuf;
use std::process::ExitCode;

use tinsel_site::{BASE_URL, Config, build};

const USAGE: &str = "usage: tinsel-site [--out <dir>] [--base-url <url>]";

fn main() -> ExitCode {
    let mut cfg = Config {
        out: PathBuf::from("dist"),
        base_url: BASE_URL.to_string(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match (flag.as_str(), args.next()) {
            ("--out", Some(dir)) => cfg.out = PathBuf::from(dir),
            ("--base-url", Some(url)) => cfg.base_url = url,
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    match build(&cfg) {
        Ok(pages) => {
            println!("wrote {pages} pages to {}", cfg.out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!(
                "error: couldn't write the site to {}: {e}",
                cfg.out.display()
            );
            ExitCode::FAILURE
        }
    }
}
