// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::errors::ParseError;
use crate::tl::{Category, Definition};

const DEFINITION_SEP: char = ';';
const FUNCTIONS_SEP: &str = "---functions---";
const TYPES_SEP: &str = "---types---";

/// An iterator over [Type Language] definitions.
///
/// [Type Language]: https://core.telegram.org/mtproto/TL
pub struct TlIterator {
    contents: String,
    index: usize,
    category: Category,
}

impl TlIterator {
    pub(crate) fn new(contents: String) -> Self {
        TlIterator {
            contents,
            index: 0,
            category: Category::Types,
        }
    }
}

impl Iterator for TlIterator {
    type Item = Result<Definition, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let definition = {
            if self.index >= self.contents.len() {
                return None;
            }

            let contents = &self.contents[self.index..];
            let mut in_comment = false;
            let mut end = contents.len();

            for (i, c) in contents.chars().enumerate() {
                if contents[i..contents.len().min(i + 2)] == *"//" {
                    in_comment = true;
                } else if in_comment && c == '\n' {
                    in_comment = false;
                }

                if !in_comment && c == DEFINITION_SEP {
                    end = i;
                    break;
                }
            }

            let definition = contents[..end].trim();
            self.index = self.index + end + 1;

            definition
        };

        // Get rid of the leading separator and adjust category
        let definition = if definition.starts_with("---") {
            if let Some(definition) = definition.strip_prefix(FUNCTIONS_SEP) {
                self.category = Category::Functions;
                definition.trim()
            } else if let Some(definition) = definition.strip_prefix(TYPES_SEP) {
                self.category = Category::Types;
                definition.trim()
            } else {
                return Some(Err(ParseError::UnknownSeparator));
            }
        } else {
            definition
        };

        // Yield the fixed definition
        Some(match definition.parse::<Definition>() {
            Ok(mut d) => {
                d.category = self.category;
                Ok(d)
            }
            x => x,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParseError;

    #[test]
    fn parse_bad_separator() {
        let mut it = TlIterator::new("---foo---".into());
        assert_eq!(it.next(), Some(Err(ParseError::UnknownSeparator)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn parse_file() {
        let mut it = TlIterator::new(
            "
            //@description This is the first definition of the test
            first#1 = t; // inline comment
            second and bad;
            third#3 = t;
            // trailing comment
        ".into(),
        );

        assert_eq!(it.next().unwrap().unwrap().id, 1);
        assert!(it.next().unwrap().is_err());
        assert_eq!(it.next().unwrap().unwrap().id, 3);
        assert_eq!(it.next(), None);
    }
}
