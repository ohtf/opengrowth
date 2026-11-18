// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON written as the upstream server writes it, and JSON read with
//! upstream's error messages.
//!
//! Writing: Starlette's `JSONResponse` calls `json.dumps(content,
//! ensure_ascii=False, allow_nan=False, indent=None, separators=(",", ":"))`.
//! That is `serde_json`'s compact output except for float spelling, which
//! uses CPython `repr` here. String escaping already agrees: both escape `"`,
//! `\`, `\b`, `\f`, `\n`, `\r`, `\t`, and other control characters as
//! lowercase `\u00XX`, and leave everything else, including non-ASCII, as is.
//!
//! Reading: `serde_json` decides validity. Only when it rejects a body is
//! [`python_decode_error`] used to find the message and character position
//! CPython's `json` module would report, which the upstream API returns.

use std::io;

use crate::pyfmt::python_repr;
use serde::Serialize;
use serde_json::ser::Formatter;

/// Compact formatting with CPython float spelling.
#[derive(Debug, Clone, Copy, Default)]
struct PythonFormatter;

impl Formatter for PythonFormatter {
    fn write_f64<W: ?Sized + io::Write>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        writer.write_all(python_repr(value).as_bytes())
    }

    fn write_f32<W: ?Sized + io::Write>(&mut self, writer: &mut W, value: f32) -> io::Result<()> {
        writer.write_all(python_repr(f64::from(value)).as_bytes())
    }
}

/// Serialises `value` exactly as the upstream server's `JSONResponse` would.
///
/// Non-finite floats become `null` (serde's behaviour); upstream raises and
/// returns a 500 instead. No response type here can hold one.
#[must_use]
pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Vec<u8> {
    let mut out = Vec::with_capacity(4096);
    let mut serializer = serde_json::Serializer::with_formatter(&mut out, PythonFormatter);
    value
        .serialize(&mut serializer)
        .expect("response types always serialise");
    out
}

/// A body CPython's `json.loads` would reject: the message and the
/// character (not byte) position it reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    /// For example "Expecting value".
    pub message: String,
    /// Zero-based character index.
    pub position: usize,
}

/// The error CPython 3.13's `json.loads` reports for `text`, or `None` if
/// CPython would accept it (for example `NaN`, which `serde_json` rejects).
#[must_use]
pub fn python_decode_error(text: &str) -> Option<DecodeError> {
    let chars: Vec<char> = text.chars().collect();
    let parser = Decoder { chars: &chars };
    let fail = |message: &str, position| {
        Some(DecodeError {
            message: message.to_owned(),
            position,
        })
    };
    let start = parser.skip_whitespace(0);
    match parser.value(start) {
        Err(error) => Some(error),
        Ok(end) => {
            let end = parser.skip_whitespace(end);
            if end == chars.len() {
                None
            } else {
                fail("Extra data", end)
            }
        }
    }
}

struct Decoder<'a> {
    chars: &'a [char],
}

type Step = Result<usize, DecodeError>;

impl Decoder<'_> {
    fn error(message: impl Into<String>, position: usize) -> DecodeError {
        DecodeError {
            message: message.into(),
            position,
        }
    }

    fn at(&self, i: usize) -> Option<char> {
        self.chars.get(i).copied()
    }

    fn skip_whitespace(&self, mut i: usize) -> usize {
        while matches!(self.at(i), Some(' ' | '\t' | '\n' | '\r')) {
            i += 1;
        }
        i
    }

    fn starts_with(&self, i: usize, word: &str) -> bool {
        word.chars()
            .enumerate()
            .all(|(k, c)| self.at(i + k) == Some(c))
    }

    fn value(&self, i: usize) -> Step {
        match self.at(i) {
            Some('"') => self.string(i),
            Some('{') => self.object(i),
            Some('[') => self.array(i),
            Some('n') if self.starts_with(i, "null") => Ok(i + 4),
            Some('t') if self.starts_with(i, "true") => Ok(i + 4),
            Some('f') if self.starts_with(i, "false") => Ok(i + 5),
            Some('N') if self.starts_with(i, "NaN") => Ok(i + 3),
            Some('I') if self.starts_with(i, "Infinity") => Ok(i + 8),
            Some('-') if self.starts_with(i, "-Infinity") => Ok(i + 9),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(i),
            _ => Err(Self::error("Expecting value", i)),
        }
    }

    /// `-?(?:0|[1-9]\d*)(\.\d+)?([eE][-+]?\d+)?`, matched as far as it goes.
    fn number(&self, start: usize) -> Step {
        let digits = |mut i: usize| {
            while self.at(i).is_some_and(|c| c.is_ascii_digit()) {
                i += 1;
            }
            i
        };
        let mut i = start;
        if self.at(i) == Some('-') {
            i += 1;
        }
        match self.at(i) {
            Some('0') => i += 1,
            Some(c) if c.is_ascii_digit() => i = digits(i),
            _ => return Err(Self::error("Expecting value", start)),
        }
        if self.at(i) == Some('.') && self.at(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            i = digits(i + 1);
        }
        if matches!(self.at(i), Some('e' | 'E')) {
            let mut j = i + 1;
            if matches!(self.at(j), Some('+' | '-')) {
                j += 1;
            }
            if self.at(j).is_some_and(|c| c.is_ascii_digit()) {
                i = digits(j);
            }
        }
        Ok(i)
    }

    fn string(&self, start: usize) -> Step {
        let mut i = start + 1;
        loop {
            match self.at(i) {
                None => return Err(Self::error("Unterminated string starting at", start)),
                Some('"') => return Ok(i + 1),
                Some('\\') => match self.at(i + 1) {
                    Some('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') => i += 2,
                    Some('u') => {
                        let hex =
                            (2..6).all(|k| self.at(i + k).is_some_and(|c| c.is_ascii_hexdigit()));
                        if !hex {
                            return Err(Self::error("Invalid \\uXXXX escape", i + 1));
                        }
                        i += 6;
                    }
                    None => return Err(Self::error("Unterminated string starting at", start)),
                    Some(_) => return Err(Self::error("Invalid \\escape", i)),
                },
                Some(c) if (c as u32) < 0x20 => {
                    return Err(Self::error("Invalid control character at", i));
                }
                Some(_) => i += 1,
            }
        }
    }

    fn object(&self, start: usize) -> Step {
        let mut i = self.skip_whitespace(start + 1);
        if self.at(i) == Some('}') {
            return Ok(i + 1);
        }
        loop {
            if self.at(i) != Some('"') {
                return Err(Self::error(
                    "Expecting property name enclosed in double quotes",
                    i,
                ));
            }
            i = self.skip_whitespace(self.string(i)?);
            if self.at(i) != Some(':') {
                return Err(Self::error("Expecting ':' delimiter", i));
            }
            i = self.skip_whitespace(i + 1);
            i = self.skip_whitespace(self.value(i)?);
            match self.at(i) {
                Some('}') => return Ok(i + 1),
                Some(',') => {
                    let comma = i;
                    i = self.skip_whitespace(i + 1);
                    if self.at(i) == Some('}') {
                        return Err(Self::error(
                            "Illegal trailing comma before end of object",
                            comma,
                        ));
                    }
                }
                _ => return Err(Self::error("Expecting ',' delimiter", i)),
            }
        }
    }

    fn array(&self, start: usize) -> Step {
        let mut i = self.skip_whitespace(start + 1);
        if self.at(i) == Some(']') {
            return Ok(i + 1);
        }
        loop {
            i = self.skip_whitespace(self.value(i)?);
            match self.at(i) {
                Some(']') => return Ok(i + 1),
                Some(',') => {
                    let comma = i;
                    i = self.skip_whitespace(i + 1);
                    if self.at(i) == Some(']') {
                        return Err(Self::error(
                            "Illegal trailing comma before end of array",
                            comma,
                        ));
                    }
                }
                _ => return Err(Self::error("Expecting ',' delimiter", i)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_use_python_spelling() {
        let value = serde_json::json!({"a": 115.0, "b": 1e-5, "c": 2, "d": "kg/m² \n"});
        assert_eq!(
            String::from_utf8(to_vec(&value)).unwrap(),
            r#"{"a":115.0,"b":1e-05,"c":2,"d":"kg/m² \n"}"#
        );
    }

    #[test]
    fn decode_errors_match_cpython() {
        let error = |text| python_decode_error(text).map(|e| (e.message, e.position));
        assert_eq!(error(r#"{"a":"#), Some(("Expecting value".to_owned(), 5)));
        assert_eq!(
            error(r#"{"sex": "male",}"#),
            Some(("Illegal trailing comma before end of object".to_owned(), 14))
        );
        assert_eq!(error("[1, 2] x"), Some(("Extra data".to_owned(), 7)));
        assert_eq!(error(r#"{"a": NaN}"#), None);
    }
}
