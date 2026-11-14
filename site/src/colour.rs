//! Making a creature's colours readable on the site's dark background.

use terminal_of_tinsel::render::parse_hex;

/// The page background, as in `static/style.css`.
pub const BACKGROUND: &str = "#0D1B18";

/// The contrast the accessibility guidelines ask of body text.
const ENOUGH: f64 = 4.5;

/// The colour, lightened towards white until it can be read on the
/// background. A colour that can't be parsed becomes the site's gold.
pub fn readable(hex: &str) -> String {
    let Some((r, g, b)) = parse_hex(hex) else {
        return "#E9C46A".to_string();
    };
    let mut rgb = [f64::from(r), f64::from(g), f64::from(b)];
    for _ in 0..40 {
        if contrast_of(rgb, channels(BACKGROUND)) >= ENOUGH {
            break;
        }
        for channel in &mut rgb {
            *channel += (255.0 - *channel) * 0.1;
        }
    }
    format!(
        "#{:02X}{:02X}{:02X}",
        rgb[0].round() as u8,
        rgb[1].round() as u8,
        rgb[2].round() as u8
    )
}

/// The contrast ratio between two colours, from 1 (none) to 21.
pub fn contrast(a: &str, b: &str) -> f64 {
    contrast_of(channels(a), channels(b))
}

fn channels(hex: &str) -> [f64; 3] {
    let (r, g, b) = parse_hex(hex).unwrap_or((0, 0, 0));
    [f64::from(r), f64::from(g), f64::from(b)]
}

fn contrast_of(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Relative luminance, as the guidelines define it.
fn luminance(rgb: [f64; 3]) -> f64 {
    let linear = |v: f64| {
        let v = v / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
}
