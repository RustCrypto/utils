//! Provides an Iterator<Item=u8> decorator that uses a finite state machine to exclude comments
//! from a string in linear time and constant space.

use std::iter::Peekable;

pub(crate) trait Exclude: Sized + Iterator<Item = u8> {
    fn exclude_comments(self) -> ExcludingComments<Self>;
}

impl<T: Iterator<Item = u8>> Exclude for T {
    fn exclude_comments(self) -> ExcludingComments<T> {
        ExcludingComments::new_from_iter(self)
    }
}

pub(crate) struct ExcludingComments<I: Iterator<Item = u8>> {
    state: State,
    iter: Peekable<I>,
}

impl<I: Iterator<Item = u8>> Iterator for ExcludingComments<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let next_byte = self.next_byte();
        if next_byte.is_none() {
            match self.state {
                State::BlockComment | State::PotentiallyLeavingBlockComment => {
                    panic!("block comment not terminated with */")
                }
                _ => {}
            }
        }
        next_byte
    }
}

/// States of the comment removal machine:
/// <pre>
///           Normal
///            '/'                   
///      PotentialComment
///     '/'            '*'
/// LineComment     BlockComment
///    '\n'            '*'
///   Normal      PotentiallyLeavingBlockComment
///                    '/'           '_'
///                   Normal     BlockComment
/// </pre>                                                  
enum State {
    Normal,
    PotentialComment { previous: u8 },
    LineComment,
    BlockComment,
    PotentiallyLeavingBlockComment,
}

impl<I: Iterator<Item = u8>> ExcludingComments<I> {
    fn new_from_iter(iter: I) -> Self {
        Self {
            state: State::Normal,
            iter: iter.peekable(),
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        loop {
            return match self.state {
                State::Normal => {
                    let next = self.iter.next()?;
                    match next {
                        b'/' => {
                            self.state = State::PotentialComment { previous: next };
                            continue;
                        }
                        _ => Some(next),
                    }
                }
                State::PotentialComment { previous } => {
                    let peeked_next = self.iter.peek()?;
                    match peeked_next {
                        b'/' => {
                            // second /, enter line comment and consume
                            self.iter.next();
                            self.state = State::LineComment;
                            continue;
                        }
                        b'*' => {
                            /* entering a block comment consume '*' */
                            self.iter.next();
                            self.state = State::BlockComment;
                            continue;
                        }
                        _ => {
                            // here we need to emit the previous character (the first '/')
                            // and do not consume the current character
                            self.state = State::Normal;
                            return Some(previous);
                        }
                    }
                }
                State::LineComment => {
                    let next = self.iter.next()?;
                    match next {
                        b'\n' => {
                            self.state = State::Normal;
                            return Some(next);
                        }
                        _ => {
                            // ignore all other characters while in the line comment
                            continue;
                        }
                    }
                }
                State::BlockComment => {
                    let next = self.iter.next()?;
                    match next {
                        b'*' => {
                            self.state = State::PotentiallyLeavingBlockComment;
                            continue;
                        }
                        _ => {
                            /* ignore all other characters while in the block comment */
                            continue;
                        }
                    }
                }
                State::PotentiallyLeavingBlockComment => {
                    let next = self.iter.next()?;
                    match next {
                        b'/' => {
                            /* Left the block comment */
                            self.state = State::Normal;
                            continue;
                        }
                        _ => {
                            /* we're still in the block comment */
                            self.state = State::BlockComment;
                            continue;
                        }
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec::IntoIter;

    use super::*;

    /// Converts the input to an iterator of u8, excludes comments, maps back to char and collects
    /// the results.
    fn exclude_comments(input: &str) -> String {
        let excluding_comments: ExcludingComments<IntoIter<u8>> = input
            .to_string()
            .into_bytes()
            .into_iter()
            .exclude_comments();
        excluding_comments.map(|b| b as char).collect()
    }

    #[test]
    fn empty() {
        assert!(exclude_comments("").is_empty());
    }

    #[test]
    fn single_char() {
        assert_eq!(exclude_comments("0"), "0");
    }

    #[test]
    fn two_chars() {
        assert_eq!(exclude_comments("ab"), "ab");
    }

    #[test]
    fn comment() {
        assert_eq!(exclude_comments("ab//cd"), "ab");
    }

    #[test]
    fn comments_are_ended_by_new_line() {
        assert_eq!(exclude_comments("ab//comment\nde"), "ab\nde");
    }

    #[test]
    fn new_lines_without_comments() {
        assert_eq!(exclude_comments("ab\nde"), "ab\nde");
    }

    #[test]
    fn single_slash_is_not_excluded() {
        assert_eq!(exclude_comments("ab/cd"), "ab/cd");
    }

    #[test]
    fn line_comments_on_multiple_lines() {
        assert_eq!(
            exclude_comments(
                "
line 1 //comment 1
line 2 // comment 2 // comment 3
line 3
line 4 // comment 4"
            ),
            "
line 1 
line 2 
line 3
line 4 "
        );
    }

    #[test]
    fn block_comment() {
        assert_eq!(exclude_comments("ab/*comment*/12"), "ab12");
    }

    #[test]
    fn empty_block_comment() {
        assert_eq!(exclude_comments("ab/**/12"), "ab12");
    }

    #[test]
    fn block_comment_with_asterisk_and_slash_inside() {
        assert_eq!(exclude_comments("ab/*false * asterisk and / */12"), "ab12");
    }

    #[test]
    fn block_comment_within_line_comment() {
        assert_eq!(exclude_comments("ab// /*comment*/12"), "ab");
    }

    #[test]
    #[should_panic(expected = "block comment not terminated with */")]
    fn block_comment_not_terminated() {
        exclude_comments("ab /*comment");
    }

    #[test]
    #[should_panic(expected = "block comment not terminated with */")]
    fn block_comment_not_completely_terminated() {
        exclude_comments("ab /*comment*");
    }

    #[test]
    fn block_and_line_comments_on_multiple_lines() {
        assert_eq!(
            exclude_comments(
                "
line 1 /* comment 1 */
line /* comment 2 */2 // line comment 1
line 3 /* some comments
over multiple lines
*/
line 4 /* more multiline comments
* with leading
* asterisks
*/end// line comment 2"
            ),
            "
line 1 
line 2 
line 3 
line 4 end"
        );
    }
}
