//! Text processing utilities for the colonnade crate.

use unicode_segmentation::UnicodeSegmentation;

#[cfg(feature = "nbsp")]
use lazy_static::lazy_static;
#[cfg(feature = "nbsp")]
use regex::Regex;

/// Split text into words, handling optional nbsp feature.
#[cfg(feature = "nbsp")]
pub fn to_words<'a>(s: &'a str) -> Vec<&'a str> {
    lazy_static! {
        static ref SPLITTABLE_SPACE: Regex = Regex::new(r"[\s&&[^\u00A0]]+").unwrap();
    }
    SPLITTABLE_SPACE
        .split(s)
        .filter(|s| !s.is_empty())
        .collect::<Vec<&'a str>>()
}

/// Split text into words, default implementation without nbsp support.
#[cfg(not(feature = "nbsp"))]
pub fn to_words<'a>(s: &'a str) -> Vec<&'a str> {
    s.split_whitespace().filter(|s| !s.is_empty()).collect::<Vec<&'a str>>()
}

/// Calculate the visual width of a string in characters (grapheme clusters).
pub fn true_width(s: &str) -> usize {
    UnicodeSegmentation::graphemes(s, true).count()
}

/// Find the longest word (by visual width) in a string.
pub fn longest_word(s: &str) -> usize {
    to_words(s)
        .iter()
        .fold(0, |acc, word| acc.max(true_width(word)))
}

/// Calculate the width of a string after whitespace normalization.
pub fn width_after_normalization(s: &str) -> usize {
    let mut length = 0;
    for word in to_words(s) {
        if length != 0 {
            length += 1; // space between words
        }
        length += true_width(word);
    }
    length
}
