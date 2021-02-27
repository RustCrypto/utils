//! OID parser with `const` support

use crate::{Arc, ObjectIdentifier, MAX_ARCS};

/// Const-friendly OID parser.
///
/// Parses an OID from the dotted string representation.
pub(crate) struct Parser {
    /// Parsed arcs in progress
    arcs: [Arc; MAX_ARCS],

    /// Current arc being parsed
    cursor: usize,
}

impl Parser {
    /// Parse an OID from a dot-delimited string e.g. `1.2.840.113549.1.1.1`
    pub(crate) const fn parse(s: &str) -> Self {
        let bytes = s.as_bytes();
        const_assert!(!bytes.is_empty(), "OID string is empty");
        const_assert!(
            matches!(bytes[0], b'0'..=b'9'),
            "OID must start with a digit"
        );

        Self {
            arcs: [0; MAX_ARCS],
            cursor: 0,
        }
        .parse_bytes(bytes)
    }

    /// Finish parsing, returning the result
    pub(crate) const fn result(self) -> ObjectIdentifier {
        let arcs = self.arcs;

        // TODO(tarcieri): refactor this!
        // This is a temporary workaround to allow this function to reuse the
        // existing validation logic in `ObjectIdentifier::new`.
        // In the next breaking release we can replace `new` with this function
        // entirely and consolidate the validation logic.
        match self.cursor {
            3 => ObjectIdentifier::new(&[arcs[0], arcs[1], arcs[2]]),
            4 => ObjectIdentifier::new(&[arcs[0], arcs[1], arcs[2], arcs[3]]),
            5 => ObjectIdentifier::new(&[arcs[0], arcs[1], arcs[2], arcs[3], arcs[4]]),
            6 => ObjectIdentifier::new(&[arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5]]),
            7 => ObjectIdentifier::new(&[
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
            ]),
            8 => ObjectIdentifier::new(&[
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5], arcs[6], arcs[7],
            ]),
            9 => ObjectIdentifier::new(&[
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5], arcs[6], arcs[7], arcs[8],
            ]),
            10 => ObjectIdentifier::new(&[
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5], arcs[6], arcs[7], arcs[8],
                arcs[9],
            ]),
            11 => ObjectIdentifier::new(&[
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5], arcs[6], arcs[7], arcs[8],
                arcs[9], arcs[10],
            ]),
            12 => ObjectIdentifier::new(&[
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4], arcs[5], arcs[6], arcs[7], arcs[8],
                arcs[9], arcs[10], arcs[11],
            ]),
            _ => ObjectIdentifier::new(&[]),
        }
    }

    /// Parse the remaining bytes
    const fn parse_bytes(mut self, bytes: &[u8]) -> Self {
        match bytes {
            [] => {
                self.cursor += 1;
                self
            }
            [byte @ b'0'..=b'9', remaining @ ..] => {
                let current_arc = self.arcs[self.cursor];
                self.arcs[self.cursor] = current_arc * 10 + parse_ascii_digit(*byte);
                self.parse_bytes(remaining)
            }
            [b'.', remaining @ ..] => {
                const_assert!(!remaining.is_empty(), "invalid trailing '.' in OID");
                const_assert!(
                    self.cursor < MAX_ARCS,
                    "maximum number of OID arcs exceeded"
                );
                self.cursor += 1;
                self.parse_bytes(remaining)
            }
            [byte, ..] => {
                const_assert!(
                    matches!(byte, b'0'..=b'9' | b'.'),
                    "invalid character in OID"
                );

                // Unreachable (checked by above `const_assert!`)
                // Needed for match exhaustiveness and matching types
                self
            }
        }
    }
}

/// Parse a digit from an ASCII character
// TODO(tarcieri): replace this with `byte.saturating_sub(b'0')` when MSRV 1.47+
const fn parse_ascii_digit(char: u8) -> Arc {
    match char {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        other => {
            const_assert!(matches!(other, b'0'..=b'9'), "invalid ASCII digit");
            0 // Unreachable due to above `const_assert`
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn parse() {
        let oid = Parser::parse("1.23.456").result();
        assert_eq!(oid, "1.23.456".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn reject_empty_string() {
        Parser::parse("");
    }

    #[test]
    #[should_panic]
    fn reject_non_digits() {
        Parser::parse("X");
    }

    #[test]
    #[should_panic]
    fn reject_trailing_dot() {
        Parser::parse("1.23.");
    }
}
