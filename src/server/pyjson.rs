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
//! Reading: a body's bytes are decoded as `json.loads` decodes them
//! ([`decode_json_bytes`]), then `serde_json` decides validity. Only when it
//! rejects a body is [`python_decode_error`] used to find the message and
//! character position CPython's `json` module would report, which the
//! upstream API returns.

use std::borrow::Cow;
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

/// The text CPython's `json.loads` reads from `bytes`: UTF-8, UTF-16, or
/// UTF-32, chosen by a byte-order mark, which is dropped, or else by where
/// the zero bytes are (`json.detect_encoding`). `None` where CPython cannot
/// decode them, and also for lone surrogates, which CPython passes through
/// but a Rust string cannot hold.
#[must_use]
pub fn decode_json_bytes(bytes: &[u8]) -> Option<Cow<'_, str>> {
    let (encoding, bom) = detect_encoding(bytes);
    let bytes = &bytes[bom..];
    match encoding {
        Encoding::Utf8 => std::str::from_utf8(bytes).ok().map(Cow::Borrowed),
        Encoding::Utf16 { little_endian } => {
            let (pairs, truncated) = bytes.as_chunks::<2>();
            if !truncated.is_empty() {
                return None;
            }
            let units = pairs.iter().map(|&pair| {
                if little_endian {
                    u16::from_le_bytes(pair)
                } else {
                    u16::from_be_bytes(pair)
                }
            });
            char::decode_utf16(units)
                .collect::<Result<String, _>>()
                .ok()
                .map(Cow::Owned)
        }
        Encoding::Utf32 { little_endian } => {
            let (quads, truncated) = bytes.as_chunks::<4>();
            if !truncated.is_empty() {
                return None;
            }
            quads
                .iter()
                .map(|&quad| {
                    char::from_u32(if little_endian {
                        u32::from_le_bytes(quad)
                    } else {
                        u32::from_be_bytes(quad)
                    })
                })
                .collect::<Option<String>>()
                .map(Cow::Owned)
        }
    }
}

/// An encoding `json.detect_encoding` can choose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Utf8,
    Utf16 { little_endian: bool },
    Utf32 { little_endian: bool },
}

/// `json.detect_encoding`, with the length of the byte-order mark to skip.
fn detect_encoding(bytes: &[u8]) -> (Encoding, usize) {
    const UTF16_LE: Encoding = Encoding::Utf16 {
        little_endian: true,
    };
    const UTF16_BE: Encoding = Encoding::Utf16 {
        little_endian: false,
    };
    const UTF32_LE: Encoding = Encoding::Utf32 {
        little_endian: true,
    };
    const UTF32_BE: Encoding = Encoding::Utf32 {
        little_endian: false,
    };
    if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        return (UTF32_BE, 4);
    }
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        return (UTF32_LE, 4);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return (UTF16_BE, 2);
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return (UTF16_LE, 2);
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (Encoding::Utf8, 3);
    }
    let encoding = match bytes {
        [0, 0, _, _, ..] => UTF32_BE,
        [0, _, _, _, ..] | [0, _] => UTF16_BE,
        [_, 0, 0, 0, ..] => UTF32_LE,
        [_, 0, _, _, ..] | [_, 0] => UTF16_LE,
        _ => Encoding::Utf8,
    };
    (encoding, 0)
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
    fn bodies_are_decoded_as_json_loads_decodes_bytes() {
        let text = "{\"a\": \"caf\u{e9}\"}";
        let utf16_le: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let utf16_be: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        let utf32_le: Vec<u8> = text
            .chars()
            .flat_map(|c| u32::from(c).to_le_bytes())
            .collect();
        let marked = |bom: &[u8], body: &[u8]| [bom, body].concat();
        for bytes in [
            text.as_bytes().to_vec(),
            marked(&[0xEF, 0xBB, 0xBF], text.as_bytes()),
            utf16_le.clone(),
            utf16_be.clone(),
            marked(&[0xFF, 0xFE], &utf16_le),
            marked(&[0xFE, 0xFF], &utf16_be),
            utf32_le.clone(),
            marked(&[0xFF, 0xFE, 0, 0], &utf32_le),
        ] {
            assert_eq!(
                decode_json_bytes(&bytes).as_deref(),
                Some(text),
                "{bytes:02x?}"
            );
        }
        // Two bytes, one of them zero, are one UTF-16 character.
        assert_eq!(decode_json_bytes(b"1\0").as_deref(), Some("1"));
        assert_eq!(decode_json_bytes(b"\x001").as_deref(), Some("1"));
        // Only the first byte-order mark is dropped.
        assert_eq!(
            decode_json_bytes("\u{feff}\u{feff}1".as_bytes()).as_deref(),
            Some("\u{feff}1")
        );
        // What CPython cannot decode: a truncated code unit, a code point
        // beyond Unicode, and bytes that are not UTF-8.
        assert_eq!(decode_json_bytes(&utf16_le[..utf16_le.len() - 1]), None);
        assert_eq!(
            decode_json_bytes(&marked(&[0xFF, 0xFE, 0, 0], &0x0011_0000_u32.to_le_bytes())),
            None
        );
        assert_eq!(decode_json_bytes(b"{\"a\": \"caf\xe9\"}"), None);
    }

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
