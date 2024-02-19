// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::ops::{Bound, RangeBounds};

// I liked this code; from old thread
// https://users.rust-lang.org/t/how-to-get-a-substring-of-a-string/1351/11

// FIXME remove dependency on this functionality only used once ...
pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn slice(&self, range: impl RangeBounds<usize>) -> &str;
    fn split_at_char(&self, index: usize) -> (&str, &str);
}

impl StringUtils for str {
    /// # substring()
    ///
    /// Returns a string that starts with first character pointed to by start.
    ///
    /// It will copy 'len' characters in the output. If the original string is
    /// nog long enough to have the remaining 'len' characters, then only the
    /// string with the remaining length will be returned. No error!
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start {
                break;
            }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            } else {
                break;
            }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len {
                break;
            }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            } else {
                break;
            }
        }
        &self[byte_start..byte_end]
    }

    /// Returns a string that starts with first character pointed to by start.
    /// The Start, Length pair is given to the input as type `RangBounds<usize>`
    ///
    /// It will copy 'len' characters in the output. If the original string is
    /// nog long enough to have the remaining 'len' characters, then only the
    /// string with the remaining length will be returned. No error!
    fn slice(&self, range: impl RangeBounds<usize>) -> &str {
        let start = match range.start_bound() {
            Bound::Included(bound) | Bound::Excluded(bound) => *bound,
            Bound::Unbounded => 0,
        };
        let len = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.len(),
        } - start;
        self.substring(start, len)
    }

    /// Splits the string into 2 sub strings: left, right.
    ///
    /// The right string starts with as first character the character the 'index'
    /// of the original string.
    fn split_at_char(&self, index: usize) -> (&str, &str) {
        let first = self.substring(0, index);
        let second = self.substring(index, self.len() - 2 - index);
        (first, second)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let s = "abcdèfghij";
        // All three statements should print:
        // "abcdè, abcdèfghij, dèfgh, dèfghij."
        println!(
            "{}, {}, {}, {}.",
            s.substring(0, 5),
            s.substring(0, 50),
            s.substring(3, 5),
            s.substring(3, 50)
        );
        println!(
            "{}, {}, {}, {}.",
            s.slice(..5),
            s.slice(..50),
            s.slice(3..8),
            s.slice(3..)
        );
        println!(
            "{}, {}, {}, {}.",
            s.slice(..=4),
            s.slice(..=49),
            s.slice(3..=7),
            s.slice(3..)
        );

        let s2 = "Können sie mir hören?";
        let (s3, s4) = s2.split_at_char(6);
        println!("{}, and: {}", s3, s4);
        assert_eq!(s3, "Können");
        assert_eq!(s4, " sie mir hören?");
    }
}
