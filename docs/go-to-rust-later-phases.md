# From Go to Rust: phases 5 to 8

The second half of [go-to-rust.md](go-to-rust.md): what packs and completion, the roster, repo polish
and the website taught about Go compared with Rust.

## Phase 5: packs, completion and man pages

- **One loader for two places.** Go's `fs.FS` is why Terror's loader doesn't care whether a pack is
  embedded or on disk: `embed.FS` and `os.DirFS` both satisfy it. Rust's standard library has no such
  interface, and `include_dir` and `std::fs` share nothing, so Tinsel defines the two methods it needs
  as a trait (`PackFiles`: read a file, list the JSON files) and implements it twice. Twenty lines buys
  what Go gives away, and nothing more than is used.
- **Lenient parsing is opt-in.** A Go struct unmarshals from `{}` without complaint. A serde struct
  fails on the first missing field unless it carries `#[serde(default)]`, which needs `Default` on
  every nested type. Tinsel turns it on for `Creature` so a community creature can be three lines, and
  then has to hold the built-in creatures to the full set in the data tests, as Terror does.
- **Completion that asks the program.** Cobra's generated scripts call back into the binary's hidden
  `__complete` command, and a `ValidArgsFunction` supplies the names. clap's stable generator writes a
  static script that only knows the flags. Its newer `CompleteEnv` works the Cobra way: the script
  runs the binary with `COMPLETE=bash`, and `ArgValueCandidates` supplies creature and pack names. It
  sits behind clap_complete's `unstable-dynamic` feature, so the scripts may change with an upgrade;
  the committed copies in `completions/` make any change show up in a diff.
- **Man pages.** Cobra's `doc.GenManTree` and clap's `clap_mangen` do the same job from the command
  tree. Terror runs its generator in a GoReleaser `before` hook. cargo-dist has no such hook, so Tinsel
  commits the output and a test compares it with a fresh run. `clap_mangen` is a dev-dependency: it
  never reaches the shipped binary.
- **Validating a flag late.** With only built-in packs, clap could reject `--pack elves` itself, with
  the choices in `--help`. Community packs aren't known until the config directory has been read, so
  the check moved to run time, as in Terror, and the "did you mean" had to be written by hand.
- **JSON key order.** Terror builds the `packs new` example from a `map[string]any`, and Go marshals
  map keys in alphabetical order, so `aliases` comes first and `name` is buried. serde_json's `json!`
  macro does the same unless its `preserve_order` feature is on. Tinsel writes the example from a text
  template instead, so the file reads top to bottom the way a person fills it in.

## Phase 6: the roster

No new code, which is the point: eight creatures went in as JSON and text files, and the commands,
the quiz, the games, completion and the man pages picked them up without a change.

- **The data tests are the reviewer.** Both projects test their data, but the split differs. In Go a
  missing field is silently the zero value, so Terror's tests have to look for emptiness. In Rust the
  parser could have refused it, but Tinsel gave every field a default for community packs in phase 5,
  so its tests do the same job as Terror's. The strictness lives in `cargo test`, not in the types.
- **Hard-coded counts are a smell in either language.** Adding creatures broke a dozen tests that said
  14. Most now ask the vault how many there are. The ones that still say 22 are end-to-end tests of the
  shipped roster, where a surprise is worth a failure.
- **Embedding is a build step.** `include_dir!` and `//go:embed` both read the files at compile time, so
  a new JSON file needs a rebuild to appear. Go's build cache notices a new file under an embedded
  directory. With `include_dir` on stable Rust, Cargo is only told about the files that were there at
  the last build, so if a new creature doesn't show up, touch `src/creatures.rs` or run `cargo clean`.

## Phase 7: repo polish

- **Releases: GoReleaser against cargo-dist.** GoReleaser is one YAML file you write: builds, archives,
  a `before` hook, the Homebrew cask, the release notes. cargo-dist is a few lines of TOML and a
  workflow it generates and owns. It gives installers (shell, PowerShell, Homebrew) for nothing, but
  where GoReleaser has a hook for everything, cargo-dist has none, which is why Tinsel commits its man
  pages instead of generating them at release time.
- **Versions.** Go stamps the version in at link time (`-ldflags -X`), from the tag. Cargo reads it from
  `Cargo.toml`, so a release is a commit that changes the version before the tag, and anything that
  embeds it (the man pages) has to be regenerated in that commit.
- **Formatting and linting.** `gofmt` and `go vet` come with Go, and nobody argues with them. `rustfmt`
  and `clippy` come with rustup and play the same parts, but clippy has far more opinions: several of
  the review fixes in this project were clippy's (`is_multiple_of`, collapsed `if let` chains).
- **One complexity budget, two languages.** slopstopper's complexity check uses the same ceiling for
  Go and Rust. Terror's Cobra commands are a file each, so they passed. Tinsel's `match` on a clap enum
  had grown into one 460-line function, and had to be split into a method per command. Rust's `match`
  makes a long function easy to write and the compiler never complains about its length.
- **Dependency updates.** Dependabot knows `gomod` and `cargo` alike. The difference is in what a bump
  can break: Go's minimal version selection changes little, while a Cargo bump can bring new versions
  of everything beneath it, so `Cargo.lock` is committed and CI builds with `--locked`.
- **Recording demos.** VHS is a Go program and installs with `go install`, even for a Rust project.

## Phase 8: the website

- **One module, or a workspace.** Terror's site is `go run ./site`: another `main` package in the same
  module, importing `internal/monsters`. Go needs nothing more. A Rust binary crate can't be imported,
  so Tinsel's modules moved behind a `src/lib.rs` and the site became a second crate in a Cargo
  workspace. `main.rs` now reaches its own modules with `use terminal_of_tinsel::...`, and the modules
  didn't change at all, because `crate::` inside them means the library either way.
- **Everything turned `pub`.** Go's `internal/` keeps a package private to the module however many
  programs it has. Rust has no equivalent between crates: for the site to use a function it has to be
  public in the library, and from then on the compiler stops warning when it's unused.
- **Templates.** Go ships `html/template`, which escapes by context. Rust's standard library has no
  templates, so the choice was a new dependency or `format!` and an `esc` function. The site uses
  `format!`: fewer dependencies, but escaping is now something a test has to hold, not the type system.
- **Raw strings and HTML.** `r#"..."#` ends at the first `"#`, and `href="#main"` contains one. The
  HTML literals are `r##"..."##`. Go's backtick strings have no such trap.
- **The command table writes itself.** The home page lists every command by asking clap for its
  subcommands, the same way the man pages are generated, so the site can't fall out of step with the
  program. Terror keeps that table by hand.
- **Workspace commands.** `go test ./...` covers every package. `cargo test` covers only the root
  package of a workspace unless told `--workspace`, which CI now is.
