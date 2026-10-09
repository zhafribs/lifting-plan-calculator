//! Number formatting shared by every screen and by the printed report.
//!
//! A port of `Figures.kt` from the Android source. One place decides how a
//! figure reads, so the phone result card and the printed plan cannot drift:
//! thousands separated, a fixed number of decimals, a dot decimal separator
//! (never the machine's locale), and an em dash for "nothing to show".

/// What a figure that has not been entered shows as.
pub const NOTHING: &str = "\u{2014}";

/// Format a figure, or return an em dash when there is nothing to show.
///
/// `None` is *not* zero: the report prints an em dash for an absent figure and
/// `0.00` for a measured zero, and the two mean different things on a plan.
pub fn fmt(value: Option<f64>, decimals: usize, unit: &str) -> String {
    match value {
        None => NOTHING.to_string(),
        Some(value) => {
            let text = grouped(value, decimals);
            if unit.is_empty() {
                text
            } else {
                format!("{text} {unit}")
            }
        }
    }
}

/// A fixed-decimals figure with thousands separators, matching the reference's
/// `"{:,.Nf}"` (Rust's standard formatter has no grouping flag).
pub fn grouped(value: f64, decimals: usize) -> String {
    let formatted = format!("{value:.decimals$}");
    let (sign, rest) = match formatted.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", formatted.as_str()),
    };
    let (integer, fraction) = match rest.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (rest, None),
    };
    let mut out = String::new();
    let bytes = integer.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && (bytes.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(*byte as char);
    }
    match fraction {
        Some(fraction) => format!("{sign}{out}.{fraction}"),
        None => format!("{sign}{out}"),
    }
}

/// A figure with a dot decimal separator, no thousands grouping.
///
/// The Python `"%.3f"` shape, used where a value is substituted into a formula
/// rather than shown in a card.
pub fn fixed(value: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, value)
}

/// A figure with thousands separators, no unit.
pub fn plain(value: f64, decimals: usize) -> String {
    fmt(Some(value), decimals, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_an_em_dash() {
        assert_eq!(fmt(None, 3, "kg"), "\u{2014}");
    }

    #[test]
    fn thousands_are_separated() {
        assert_eq!(fmt(Some(1234567.891), 2, "kg"), "1,234,567.89 kg");
    }

    #[test]
    fn zero_is_a_measured_zero() {
        assert_eq!(fmt(Some(0.0), 3, "m"), "0.000 m");
    }

    #[test]
    fn fixed_has_no_grouping() {
        assert_eq!(fixed(1234.5, 1), "1234.5");
    }
}
