# Night 14 · Whispers in the Snow

> 📺 *"A viewer writes: 'Dear Auntie, which of your guests was it that wanted porridge? I only caught the end.' Well, I'm not reading you all twenty-two again. Tonight you press one key, whisper a word into the snow, and the right guest comes to the door. Spelling optional. Accents very optional."*

**Tonight you'll learn**
- A mode: one flag that changes what every key means
- Text input with nothing but a `String`: `push` and `pop`
- Why Rust strings are searched as `char`s here, not bytes
- Iterator tools: `windows`, `position`, `find_map`, `filter_map`, and `?` on an `Option`
- Ranking by the order you look in
- Cutting a snippet around a match

**Where we are:** the explorer has an intro, a gallery and pages with sections. To find the guest who wanted porridge you would have to read all of them.

## A mode is a flag

Last night's `on_key` had an arm we skipped. Here it is, from [`src/explorer/model.rs`](../src/explorer/model.rs):

```rust
            KeyCode::Char('/') => {
                self.searching = true;
                self.query.clear();
                self.hits.clear();
                self.hit_cursor = 0;
            }
```

Four fields of the model belong to search: whether it is on, what has been typed, what was found, and which result is selected. Pressing `/` switches it on and empties the rest. From then on, the line near the top of `on_key` sends every key somewhere else:

```rust
        if self.searching {
            return self.search_key(key.code);
        }
```

That `return` is the whole idea of a mode. While searching, `q` does not quit and `g` does not open the gallery, because the ordinary `match` is never reached. They are letters, and somebody may be looking for a goat. Only Ctrl+C is checked before this line, so it still works, and one of the tests is named for it: `ctrl_c_quits_from_anywhere_even_a_search`.

## Typing

There is no text-input widget. The query is a `String` and the keys edit it:

```rust
    fn search_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.searching = false;
                return;
            }
            KeyCode::Enter => {
                if let Some(hit) = self.hits.get(self.hit_cursor) {
                    let index = hit.index;
                    self.searching = false;
                    self.open(index);
                }
                return;
            }
            // ...
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Char(c) => self.query.push(c),
            _ => return,
        }
        self.hits = search::search(&self.vault.creatures, &self.query);
        self.hit_cursor = 0;
    }
```

`push` adds a character to the end and `pop` takes the last one off. `pop` removes a whole character however many bytes it takes, so backspacing over `ý` works. It returns an `Option<char>`, which is `None` for an empty string, so backspace on nothing is harmless.

Look at which arms end in `return`. Escape, Enter and the arrows leave early. Only the two arms that *change the query* fall out of the `match` and reach the last two lines, where the search runs again and the selection goes back to the top. So the results follow every keystroke, and nothing else triggers a search.

In the Enter arm, `self.hits.get(self.hit_cursor)` returns an `Option`, so Enter with no results does nothing. `let index = hit.index;` copies the number out before `self.open(index)` is called. `hit` is a borrow of `self.hits`, and `open` needs `&mut self`, so the borrow must be finished first. Copying one `usize` ends it.

The typed text is drawn by the footer in [`src/explorer/view.rs`](../src/explorer/view.rs), with a block for a cursor:

```rust
        let mut lines = vec![Line::from(vec![
            Span::styled("/", heading_style(m)),
            Span::raw(format!(" {}▌", m.query)),
        ])];
```

## What a result is

The searching itself is in [`src/search.rs`](../src/search.rs), which knows nothing about keys or screens:

```rust
/// A creature matching a search, with where and why it matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The creature's position in the list that was searched.
    pub index: usize,
    /// The field that matched: name, alias, fact and so on.
    pub field: &'static str,
    /// The matching text, trimmed to a window around the match.
    pub snippet: String,
}
```

A hit holds the creature's *index*, not a reference to it. A reference would tie every `Hit` to the vault with a lifetime, and the model that stores hits would need to carry it too. An index is a plain number, and `vault.creatures[hit.index]` gets the creature back when it is wanted.

## Folding, one character at a time

On Night 7 you folded accents so that `gryla` finds Grýla. Search uses the same function, from [`src/creatures.rs`](../src/creatures.rs):

```rust
/// Lowercases one character and folds common accents, always to exactly one
/// character, so positions in folded text line up with the original.
pub fn fold(ch: char) -> char {
    match ch.to_lowercase().next().unwrap_or(ch) {
        'á' | 'à' | 'â' | 'ä' => 'a',
        // ...
        'ð' => 'd',
        'þ' => 't',
        // ...
        c => c,
    }
}
```

The doc comment carries the design: one character in, exactly one out. That promise is what the search is built on.

## Finding the needle

```rust
pub fn search(creatures: &[Creature], query: &str) -> Vec<Hit> {
    let needle: Vec<char> = query.trim().chars().map(fold).collect();
    if needle.is_empty() {
        return Vec::new();
    }
    creatures
        .iter()
        .enumerate()
        .filter_map(|(index, c)| {
            fields(c).into_iter().find_map(|(field, text)| {
                let chars: Vec<char> = text.chars().collect();
                let folded: Vec<char> = chars.iter().copied().map(fold).collect();
                let at = folded.windows(needle.len()).position(|w| w == needle)?;
                Some(Hit {
                    index,
                    field,
                    snippet: snippet(&chars, at, needle.len()),
                })
            })
        })
        .collect()
}
```

Start in the middle. Each text is turned into two vectors of characters: `chars`, as written, and `folded`, lowercased and unaccented. Because `fold` is one for one, position 12 in `folded` is position 12 in `chars`. The match is found in the folded copy and the snippet is cut from the original, so a search for `gryla` shows `Grýla` in its results with the accent intact.

Why vectors of `char` at all? A Rust `String` is UTF-8 bytes, and a character takes between one and four of them: `a` is one byte, `ý` is two. Positions in a string are byte positions, and slicing in the middle of a character panics. Tonight's code wants "thirty characters either side", which is simple arithmetic on a `Vec<char>` and a hazard on a `&str`.

Now the iterator methods, from the inside out:

- `folded.windows(n)` gives every run of `n` neighbouring items: for `[k, r, a, m]` and 2, that is `[k, r]`, `[r, a]`, `[a, m]`. If the needle is longer than the text there are no windows at all, and no panic.
- `.position(|w| w == needle)` returns the index of the first window equal to the needle, as an `Option<usize>`.
- The `?` after it is the operator you know from `Result`, working on an `Option`: if there is no match, the closure returns `None` straight away.
- `find_map` runs that closure over a creature's fields and stops at the first `Some`. Each creature appears at most once.
- `filter_map` runs over the creatures and keeps only those that produced a hit.

This is a substring search, with no notion of words. Search for `the` and Père Fouettard arrives by way of his alias "Father Whipper", which has the letters in the middle.

## Ranking by the order you look in

`find_map` stops at the first field that matches, so the order of the fields is the ranking:

```rust
/// What is searched, most important first.
fn fields(c: &Creature) -> Vec<(&'static str, String)> {
    let mut out = vec![("name", c.name.clone())];
    out.extend(c.aliases.iter().map(|a| ("alias", a.clone())));
    out.push(("description", c.description.clone()));
    out.push(("origin", c.origin.clone()));
    // ...
    out.extend(c.facts.iter().map(|f| ("fact", f.clone())));
    out.push(("legend", c.legend.clone()));
    // ...
    out
}
```

There is no scoring. A creature whose name matches is reported as a name match, even if a fact mentions the word as well. Between creatures, the results stay in the order of the gallery. A few lines of Rust that call `search` on the built-in vault and print each hit's creature, field and snippet give this for three queries:

```
== porridge
The Tomte        (description) … of the farm, who expects his porridge on Christmas Eve, with butter
== GRYLA
The Yule Cat     (fact) …gs it's the pet of the ogress Grýla
Grýla            (name) Grýla
The Yule Lads    (description) Grýla's thirteen troublemaking sons…
== birch
Krampus          (fact) … chains, bells and bundles of birch switches called Ruten
```

The viewer's answer is the Tomte, the farm guardian who expects porridge on Christmas Eve, with butter. For `GRYLA`, the Yule Cat comes first because the cat comes first in the gallery, not because it is the best match. With twenty-two guests that is good enough, and it is a deliberate choice of the plain thing over the clever one. It is also fast enough to run on every keystroke with no index and no cache.

## The snippet

```rust
/// Trims long text to a window around the match.
fn snippet(chars: &[char], at: usize, len: usize) -> String {
    const PAD: usize = 30;
    let from = at.saturating_sub(PAD);
    let to = (at + len + PAD).min(chars.len());
    let mut out = String::new();
    if from > 0 {
        out.push('…');
    }
    out.extend(&chars[from..to]);
    if to < chars.len() {
        out.push('…');
    }
    out
}
```

Thirty characters each side of the match. Both ends are clamped: `saturating_sub` stops `from` at zero, and `.min(chars.len())` stops `to` at the end. An ellipsis goes on only where something was cut, which is why `Grýla` alone has none and the Tomte's line has one at the front. A `const` inside a function is visible only there.

## Showing the results

While `searching` is true, the view draws `search_results` where the gallery or the page would be. Each hit is two lines, a name and its reason:

```rust
        lines.push(Line::styled(
            format!(
                "    {}",
                truncate(
                    &format!("({}) {}", hit.field, hit.snippet),
                    width.saturating_sub(6).max(10)
                )
            ),
            dim(m),
        ));
```

Below that, the function keeps the selected hit in view with the same offset sum as last night's gallery, and the same `viewport`.

Because search is a function of a slice and a string, its tests need no screen:

```bash
cargo test search::
```

```
running 5 tests
test search::tests::film_notes_are_searched ... ok
test search::tests::names_rank_before_lore ... ok
test search::tests::snippets_are_a_window_round_the_match ... ok
test search::tests::lore_is_searched_and_each_creature_appears_once ... ok
test search::tests::case_and_accents_do_not_matter ... ok
```

The typing is tested separately, in `search_finds_lore_and_opens_the_creature` in [`src/explorer/tests.rs`](../src/explorer/tests.rs), which types `porridgx`, backspaces, types `e`, and presses Enter.

## Try it

- `cargo run -- creature --all --no-intro`, press `/` and type `the`. Find a result where the match is inside a longer word.
- Search for `pere` and for `PÈRE`. Then read `case_and_accents_do_not_matter` in `src/search.rs`.
- Press `/`, type `q`. You are still here. Now press Esc and then `q`.

## 🎄 Festive fact

`char::to_lowercase` returns an iterator, not a `char`, because lowercasing one character can produce more than one. The capital dotted İ used in Turkish (U+0130) lowercases to two: an `i` followed by a combining dot. That is why `fold` says `.next().unwrap_or(ch)`: it takes the first character and keeps its one-in, one-out promise.

## 🕯️ Before morning

A creature's `powers` and `weaknesses` are on its Stat Card, but search does not look there: `riding company`, one of Lussi's powers, finds nobody. Add both to `fields`, and write a test that proves it.

Hint: copy the `c.legacy` line at the end of `fields` twice and change the label and the field. Where you put them in the list is their rank, so decide whether a power should beat a quote. For the test, copy `film_notes_are_searched` and expect `hits[0].field` to be your new label.

> 📺 *"The Tomte, dear viewer. It was the Tomte, and do not forget the butter. Tomorrow night the cameras turn round for once, and you can see who has been talking to you all this time."*

[← Night 13](night-13.md) · [Index](README.md) · [Night 15 →](night-15.md)
