//! # camelcase — convert a string to `camelCase`
//!
//! Convert a dash/dot/underscore/space-separated string (or `snake_case`, `kebab-case`,
//! `PascalCase`, …) to `camelCase`: `foo-bar` → `fooBar`, `foo_bar_baz` → `fooBarBaz`. A
//! faithful Rust port of the widely-used
//! [`camelcase`](https://www.npmjs.com/package/camelcase) npm package.
//!
//! ```
//! use camelcase::camel_case;
//!
//! assert_eq!(camel_case("foo-bar"), "fooBar");
//! assert_eq!(camel_case("foo_bar_baz"), "fooBarBaz");
//! assert_eq!(camel_case("XML-http"), "xmlHttp");
//! ```
//!
//! Use [`camel_case_with`] for `PascalCase` or to preserve consecutive uppercase runs:
//!
//! ```
//! use camelcase::{camel_case_with, Options};
//!
//! assert_eq!(camel_case_with("foo-bar", Options::new().pascal_case(true)), "FooBar");
//! assert_eq!(
//!     camel_case_with("XML-http", Options::new().preserve_consecutive_uppercase(true)),
//!     "XMLHttp"
//! );
//! ```

#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/camelcase/0.1.0")]

use regex::{Captures, Regex};
use std::sync::OnceLock;

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// Options for [`camel_case_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pascal_case: bool,
    preserve_consecutive_uppercase: bool,
    capitalize_after_number: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            pascal_case: false,
            preserve_consecutive_uppercase: false,
            capitalize_after_number: true,
        }
    }
}

impl Options {
    /// Default options (`capitalize_after_number = true`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Uppercase the first character of the result (`PascalCase`).
    #[must_use]
    pub fn pascal_case(mut self, value: bool) -> Self {
        self.pascal_case = value;
        self
    }

    /// Keep runs of consecutive uppercase letters as-is (`XMLHttp` instead of `xmlHttp`).
    #[must_use]
    pub fn preserve_consecutive_uppercase(mut self, value: bool) -> Self {
        self.preserve_consecutive_uppercase = value;
        self
    }

    /// Whether a digit sequence forms a word boundary (so the following letter is
    /// uppercased). Defaults to `true`.
    #[must_use]
    pub fn capitalize_after_number(mut self, value: bool) -> Self {
        self.capitalize_after_number = value;
        self
    }
}

fn is_upper(c: char) -> bool {
    c.is_uppercase()
}

fn is_lower(c: char) -> bool {
    c.is_lowercase()
}

fn is_separator(c: char) -> bool {
    matches!(c, '_' | '.' | '-' | ' ')
}

fn lower(s: &str) -> String {
    s.to_lowercase()
}

fn upper(s: &str) -> String {
    s.to_uppercase()
}

fn numbers_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[0-9]+([\p{Alphabetic}\p{N}_]|$)").expect("valid"))
}

fn separators_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[_.\- ]+([\p{Alphabetic}\p{N}_]|$)").expect("valid"))
}

/// `String.prototype.trim()` whitespace set.
fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// Convert `input` to `camelCase` with the default options.
///
/// ```
/// # use camelcase::camel_case;
/// assert_eq!(camel_case("foo bar"), "fooBar");
/// ```
#[must_use]
pub fn camel_case(input: &str) -> String {
    camel_case_with(input, Options::default())
}

/// Convert a list of fragments to `camelCase` (each is trimmed, empties dropped, joined by
/// `-`).
///
/// ```
/// # use camelcase::{camel_case_slice, Options};
/// assert_eq!(camel_case_slice(&["foo", "bar"], Options::default()), "fooBar");
/// ```
#[must_use]
pub fn camel_case_slice(input: &[&str], options: Options) -> String {
    let joined = input
        .iter()
        .map(|s| s.trim_matches(is_js_whitespace))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    camel_case_with(&joined, options)
}

/// Convert `input` to `camelCase` with the given [`Options`].
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn camel_case_with(input: &str, options: Options) -> String {
    let trimmed = input.trim_matches(is_js_whitespace);
    if trimmed.is_empty() {
        return String::new();
    }

    // Preserve leading `_` and `$`.
    let leading_len = trimmed
        .chars()
        .take_while(|&c| c == '_' || c == '$')
        .map(char::len_utf8)
        .sum::<usize>();
    let leading_prefix = &trimmed[..leading_len];
    let mut work = trimmed[leading_len..].to_string();

    if work.is_empty() {
        return leading_prefix.to_string();
    }

    if work.chars().count() == 1 {
        let only = work.chars().next().unwrap();
        if is_separator(only) {
            return leading_prefix.to_string();
        }
        let cased = if options.pascal_case {
            upper(&work)
        } else {
            lower(&work)
        };
        return format!("{leading_prefix}{cased}");
    }

    let has_upper = work != lower(&work);
    if has_upper {
        work = preserve_camel_case(&work, options.preserve_consecutive_uppercase);
    }

    // Strip leading separators.
    work = strip_leading_separators(&work);

    if options.capitalize_after_number {
        work = if options.preserve_consecutive_uppercase {
            lowercase_leading_capital(&work)
        } else {
            lower(&work)
        };
    } else {
        work = process_with_case_preservation(&work, options.preserve_consecutive_uppercase);
    }

    if options.pascal_case && !work.is_empty() {
        let mut chars = work.chars();
        let first = chars.next().unwrap();
        work = format!("{}{}", upper(&first.to_string()), chars.as_str());
    }

    format!("{leading_prefix}{}", post_process(&work, options))
}

/// Insert `-` separators at case-transition boundaries (the reference's `preserveCamelCase`).
fn preserve_camel_case(input: &str, preserve_consecutive_uppercase: bool) -> String {
    let mut chars: Vec<char> = input.chars().collect();
    let mut is_last_char_lower = false;
    let mut is_last_char_upper = false;
    let mut is_last_last_char_upper = false;

    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        let is_last_last_char_preserved = if index > 2 {
            chars[index - 3] == '-'
        } else {
            true
        };

        if is_last_char_lower && is_upper(character) {
            chars.insert(index, '-');
            is_last_char_lower = false;
            is_last_last_char_upper = is_last_char_upper;
            is_last_char_upper = true;
            index += 1;
        } else if is_last_char_upper
            && is_last_last_char_upper
            && is_lower(character)
            && (!is_last_last_char_preserved || preserve_consecutive_uppercase)
        {
            chars.insert(index - 1, '-');
            is_last_last_char_upper = is_last_char_upper;
            is_last_char_upper = false;
            is_last_char_lower = true;
        } else {
            let cs = character.to_string();
            let lc = lower(&cs);
            let uc = upper(&cs);
            is_last_char_lower = lc == cs && uc != cs;
            is_last_last_char_upper = is_last_char_upper;
            is_last_char_upper = uc == cs && lc != cs;
        }

        index += 1;
    }

    chars.into_iter().collect()
}

/// `LEADING_SEPARATORS = /^[_.\- ]+/` — strip a leading run of separators.
fn strip_leading_separators(s: &str) -> String {
    s.trim_start_matches(is_separator).to_string()
}

/// `preserveConsecutiveUppercase` helper: lowercase a leading capital that is not followed
/// by another uppercase (`LEADING_CAPITAL = /^[\p{Lu}](?![\p{Lu}])/`).
fn lowercase_leading_capital(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let second_is_upper = chars.clone().next().is_some_and(is_upper);
    if is_upper(first) && !second_is_upper {
        format!("{}{}", lower(&first.to_string()), chars.as_str())
    } else {
        s.to_string()
    }
}

/// The `capitalizeAfterNumber: false` path (`processWithCasePreservation`).
fn process_with_case_preservation(input: &str, preserve_consecutive_uppercase: bool) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(input.len());
    let mut previous_was_number = false;
    let mut previous_was_uppercase = false;

    for index in 0..chars.len() {
        let character = chars[index];
        let is_upper_case = is_upper(character);
        let next_is_upper = index + 1 < chars.len() && is_upper(chars[index + 1]);

        if previous_was_number && character.is_alphabetic() {
            result.push(character);
            previous_was_number = false;
            previous_was_uppercase = is_upper_case;
        } else if preserve_consecutive_uppercase
            && is_upper_case
            && (previous_was_uppercase || next_is_upper)
        {
            result.push(character);
            previous_was_uppercase = true;
        } else if character.is_ascii_digit() {
            result.push(character);
            previous_was_number = true;
            previous_was_uppercase = false;
        } else if is_separator(character) {
            result.push(character);
            previous_was_uppercase = false;
        } else {
            result.push_str(&lower(&character.to_string()));
            previous_was_number = false;
            previous_was_uppercase = false;
        }
    }

    result
}

/// Collapse separators (and numeric word boundaries) and uppercase the following identifier.
fn post_process(input: &str, options: Options) -> String {
    let after_numbers = if options.capitalize_after_number {
        numbers_re()
            .replace_all(input, |caps: &Captures| {
                let whole = caps.get(0).unwrap();
                let m = whole.as_str();
                let identifier = caps.get(1).map_or("", |x| x.as_str());
                let next_char = input[whole.end()..].chars().next();
                if next_char.is_some_and(is_separator) {
                    return m.to_string();
                }
                if identifier.is_empty() {
                    m.to_string()
                } else {
                    format!("{}{}", &m[..m.len() - identifier.len()], upper(identifier))
                }
            })
            .into_owned()
    } else {
        input.to_string()
    };

    separators_re()
        .replace_all(&after_numbers, |caps: &Captures| {
            upper(caps.get(1).map_or("", |x| x.as_str()))
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(camel_case("foo-bar"), "fooBar");
        assert_eq!(camel_case("foo_bar"), "fooBar");
        assert_eq!(camel_case("Foo-Bar"), "fooBar");
        assert_eq!(camel_case("foo bar baz"), "fooBarBaz");
        assert_eq!(camel_case("--foo.bar"), "fooBar");
        assert_eq!(camel_case("fooBar"), "fooBar");
    }

    #[test]
    fn acronyms_and_numbers() {
        assert_eq!(camel_case("XML-http"), "xmlHttp");
        assert_eq!(camel_case("FOO-BAR"), "fooBar");
        assert_eq!(camel_case("p2p network"), "p2pNetwork");
        assert_eq!(camel_case("foo123bar"), "foo123Bar");
        assert_eq!(camel_case("fooBar123"), "fooBar123");
        assert_eq!(camel_case("ABc"), "abc");
        assert_eq!(camel_case("aBc"), "aBc");
    }

    #[test]
    fn leading_and_empty() {
        assert_eq!(camel_case("_foo"), "_foo");
        assert_eq!(camel_case("$foo_bar"), "$fooBar");
        assert_eq!(camel_case("__foo__bar__"), "__fooBar");
        assert_eq!(camel_case(""), "");
        assert_eq!(camel_case("_"), "_");
        assert_eq!(camel_case("a"), "a");
        assert_eq!(camel_case("A"), "a");
    }

    #[test]
    fn pascal_and_preserve() {
        let p = Options::new().pascal_case(true);
        assert_eq!(camel_case_with("foo-bar", p), "FooBar");
        assert_eq!(camel_case_with("p2p network", p), "P2pNetwork");

        let pres = Options::new().preserve_consecutive_uppercase(true);
        assert_eq!(camel_case_with("XML-http", pres), "XMLHttp");
        assert_eq!(camel_case_with("FOO-BAR", pres), "FOOBAR");
        assert_eq!(camel_case_with("fooBAR", pres), "fooBAR");
    }

    #[test]
    fn capitalize_after_number_false() {
        let opt = Options::new().capitalize_after_number(false);
        assert_eq!(camel_case_with("123test", opt), "123test");
        assert_eq!(camel_case_with("foo123bar", opt), "foo123bar");
        assert_eq!(camel_case_with("b2b_registration", opt), "b2bRegistration");
    }

    #[test]
    fn slice_input() {
        assert_eq!(
            camel_case_slice(&["foo", "bar"], Options::default()),
            "fooBar"
        );
        assert_eq!(
            camel_case_slice(&["  ", "foo", "", "bar"], Options::default()),
            "fooBar"
        );
    }
}
