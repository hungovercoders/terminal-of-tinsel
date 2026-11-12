//! Auntie Aerial, Terminal of Tinsel's host, broadcasting from the top of
//! the tree on Channel 25's Midwinter Special.

use crate::creatures::Creature;
use crate::rng::Rng;

/// The host's on-air name.
pub const NAME: &str = "Auntie Aerial";

/// The fictional station she broadcasts on.
pub const CHANNEL: &str = "Channel 25";

const GREETINGS: &[&str] = &[
    "Good evening, and a very merry midwinter. Welcome to the Channel 25 Midwinter Special. I'm your host, Auntie Aerial, broadcasting from the top of the tree.",
    "Pull up a chair by the fire, pour something warm and turn the lights down low. Tonight's guests have come a long way through the snow.",
    "Ah, you've tuned in. Lovely. Mind the tinsel, it's holding the aerial up.",
    "Welcome back, night owls and early stocking-checkers. The mince pies are warm and the visitors are at the door.",
];

const QUIPS: &[&str] = &[
    "Put that in the cracker jokes. It's better than the usual ones.",
    "Now you know. Tell them over the sprouts.",
    "Festive, isn't it? In a worrying sort of way.",
    "Something to think about while you're putting the star on top.",
    "And they say Christmas telly isn't educational.",
];

/// A random opening line.
pub fn greeting(rng: &mut Rng) -> &'static str {
    rng.pick(GREETINGS)
}

/// A short reaction to a fact.
pub fn quip(rng: &mut Rng) -> &'static str {
    rng.pick(QUIPS)
}

/// The host's introduction for a creature.
pub fn intro(c: &Creature) -> String {
    if c.host_intro.is_empty() {
        format!("Tonight's guest: {}. Wrap up warm.", c.name)
    } else {
        c.host_intro.clone()
    }
}
