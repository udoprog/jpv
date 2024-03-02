use std::str;

use encoding_rs::{DecoderResult, EUC_JP};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

const NUL: u8 = 0;

/// An entry.
#[borrowme::borrowme]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[musli(packed)]
pub struct Entry<'a> {
    pub kanji: &'a str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radicals: Vec<&'a str>,
}

/// A KRADFILE parser.
pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    /// Construct a new KRADFILE parser.
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    /// Step to the next byte.
    pub fn advance(&mut self) {
        self.pos = self.pos.saturating_add(1).min(self.input.len());
    }

    /// Peek the next byte.
    pub fn peek(&mut self) -> u8 {
        let Some(byte) = self.input.get(self.pos) else {
            return NUL;
        };

        *byte
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Parse an entry.
    pub fn parse(&mut self) -> Option<OwnedEntry> {
        let mut buf = [0; 2048];

        while !self.is_eof() {
            while self.peek().is_ascii_whitespace() {
                self.advance();
            }

            if self.peek() == b'#' {
                while !matches!(self.peek(), b'\n' | NUL) {
                    self.advance();
                }

                continue;
            }

            let start = self.pos;

            while !matches!(self.peek(), b'\n' | NUL) {
                self.advance();
            }

            let end = self.pos;
            self.advance();

            let mut decoder = EUC_JP.new_decoder();
            let (result, _, written) =
                decoder.decode_to_utf8_without_replacement(&self.input[start..end], &mut buf, true);

            match result {
                DecoderResult::InputEmpty => {}
                DecoderResult::OutputFull => {
                    continue;
                }
                DecoderResult::Malformed(..) => {
                    continue;
                }
            }

            let Ok(line) = str::from_utf8(&buf[..written]) else {
                continue;
            };

            let Some((kanji, remainder)) = line.split_once(" : ") else {
                continue;
            };

            let radicals = remainder.split_whitespace().map(str::to_owned).collect();
            return Some(OwnedEntry {
                kanji: kanji.to_owned(),
                radicals,
            });
        }

        None
    }
}
