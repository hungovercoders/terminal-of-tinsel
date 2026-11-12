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

const SIGN_OFFS: &[&str] = &[
    "That's all from the top of the tree. Sleep well, and leave the porridge out. Auntie Aerial, signing off.",
    "Don't touch that dial... actually, do. It's late, and somebody has to hang the stockings. Good night!",
    "The test card is coming up, which means it's time for bed. Mind the Yule Cat on the stairs.",
    "Until next time, keep your rhymes sharp and your jumper new. Good night from Channel 25.",
];

const CHRISTMAS_EVE: &str = "It's Christmas Eve, and every guest we have is out in the snow tonight. Porridge on the step, new clothes on, rhymes at the ready. I'm Auntie Aerial, and this is the biggest Midwinter Special of the year.";

const CHRISTMAS_DAY: &str = "Merry Christmas from the top of the tree! The Yule Lads are heading home one by one, and there's a mince pie with your name on it. I'm Auntie Aerial, and you're watching Channel 25.";

/// The opening line for the big nights, if this is one.
pub fn seasonal_greeting(month: i8, day: i8) -> Option<&'static str> {
    match (month, day) {
        (12, 24) => Some(CHRISTMAS_EVE),
        (12, 25) => Some(CHRISTMAS_DAY),
        _ => None,
    }
}

/// A random closing line.
pub fn sign_off(rng: &mut Rng) -> &'static str {
    rng.pick::<&str>(SIGN_OFFS)
}

/// A random opening line.
pub fn greeting(rng: &mut Rng) -> &'static str {
    rng.pick::<&str>(GREETINGS)
}

/// A short reaction to a fact.
pub fn quip(rng: &mut Rng) -> &'static str {
    rng.pick::<&str>(QUIPS)
}

/// The host's introduction for a creature.
pub fn intro(c: &Creature) -> String {
    if c.host_intro.is_empty() {
        format!("Tonight's guest: {}. Wrap up warm.", c.name)
    } else {
        c.host_intro.clone()
    }
}
