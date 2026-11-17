// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Python-compatible float text.
//!
//! The upstream API writes floats with CPython's `repr` (through `json.dumps`
//! and f-strings), so byte-identical output needs the same spelling:
//! `115.0` not `115`, `1e-05` not `1e-5`, `1e+16` not `1e16`.
//!
//! Both CPython and Rust produce the *shortest* digit string that round-trips
//! to the same `f64`, choosing the closest when several are equally short, so
//! the digits agree; only the layout differs. CPython (`format_float_short`,
//! mode `r`) uses scientific notation when the decimal point position is
//! `<= -4` or `> 16`. The `python_repr` conformance test checks this against
//! CPython for every case in `conformance/expected/python-repr.json`.

/// `repr(x)` as CPython 3 writes it.
#[must_use]
pub fn python_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".to_owned();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf" } else { "-inf" }.to_owned();
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned();
    }

    // Rust's LowerExp without a precision gives the shortest round-trip digits.
    let scientific = format!("{x:e}");
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("LowerExp output always has an exponent");
    let exponent: i32 = exponent.parse().expect("LowerExp exponent is an integer");
    let negative = mantissa.starts_with('-');
    let mut digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let mut exponent = exponent;
    if let Some((even_digits, even_exponent)) = even_tie_break(x, &digits, exponent) {
        digits = even_digits;
        exponent = even_exponent;
    }
    // Position of the decimal point relative to the start of `digits`.
    let decimal_point = exponent + 1;

    let mut out = String::with_capacity(digits.len() + 8);
    if negative {
        out.push('-');
    }
    if decimal_point <= -4 || decimal_point > 16 {
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        let shown = decimal_point - 1;
        out.push('e');
        out.push(if shown < 0 { '-' } else { '+' });
        let magnitude = shown.unsigned_abs();
        if magnitude < 10 {
            out.push('0');
        }
        out.push_str(&magnitude.to_string());
    } else if decimal_point <= 0 {
        out.push_str("0.");
        for _ in 0..(-decimal_point) {
            out.push('0');
        }
        out.push_str(&digits);
    } else {
        let point = usize::try_from(decimal_point).expect("positive");
        if point >= digits.len() {
            out.push_str(&digits);
            for _ in digits.len()..point {
                out.push('0');
            }
            out.push_str(".0");
        } else {
            out.push_str(&digits[..point]);
            out.push('.');
            out.push_str(&digits[point..]);
        }
    }
    out
}

/// When `x` lies exactly halfway between two equally short candidates,
/// CPython's dtoa chooses the one with an even last digit; Rust's shortest
/// formatter may choose the other. Returns the even candidate if this is
/// such a tie and Rust chose the odd one.
///
/// A tie needs the exact decimal expansion of `x` to be one digit longer
/// than the shortest form, at most 18 significant digits. Writing
/// `|x| = m * 2^q` with `m` odd, a negative `q` gives at least
/// `-q * log10(5)` significant digits, so ties are impossible for
/// `q < -26`. That filter keeps the exact check off the hot path: typical
/// SDS and age values have about 50 fractional bits.
fn even_tie_break(x: f64, digits: &str, exponent: i32) -> Option<(String, i32)> {
    if digits.len() < 15 || binary_exponent_of_odd_mantissa(x) < -26 {
        return None;
    }
    // Rust's fixed-precision formatting is exact; 800 digits covers every f64.
    let exact = format!("{:.800e}", x.abs());
    let (exact_mantissa, exact_exponent) = exact.split_once('e')?;
    if exact_exponent.parse::<i32>().ok()? != exponent {
        return None;
    }
    let exact_digits: Vec<u8> = exact_mantissa.bytes().filter(u8::is_ascii_digit).collect();
    let k = digits.len();
    let tail = exact_digits.get(k..)?;
    let is_tie = tail.first() == Some(&b'5') && tail[1..].iter().all(|&d| d == b'0');
    if !is_tie {
        return None;
    }
    let lower = &exact_digits[..k];
    if lower[k - 1].is_multiple_of(2) {
        let lower = String::from_utf8(lower.to_vec()).ok()?;
        return (lower != digits).then_some((lower, exponent));
    }
    // The upper candidate, with carry.
    let mut upper = lower.to_vec();
    let mut i = k;
    loop {
        if i == 0 {
            upper.insert(0, b'1');
            upper.pop();
            let upper = String::from_utf8(upper).ok()?;
            let upper = upper.trim_end_matches('0').to_owned();
            return (upper != digits).then_some((upper, exponent + 1));
        }
        i -= 1;
        if upper[i] == b'9' {
            upper[i] = b'0';
        } else {
            upper[i] += 1;
            break;
        }
    }
    let upper = String::from_utf8(upper).ok()?;
    let upper = upper.trim_end_matches('0').to_owned();
    (upper != digits).then_some((upper, exponent))
}

/// `q` in `|x| = m * 2^q` with `m` odd, for finite non-zero `x`.
fn binary_exponent_of_odd_mantissa(x: f64) -> i32 {
    let bits = x.abs().to_bits();
    let biased = i32::try_from(bits >> 52).expect("11 bits");
    let fraction = bits & ((1 << 52) - 1);
    let (mantissa, exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), biased - 1075)
    };
    exponent + i32::try_from(mantissa.trailing_zeros()).expect("at most 64")
}

#[cfg(test)]
mod tests {
    use super::python_repr;

    #[test]
    fn matches_known_cpython_spellings() {
        // Each expected string is what CPython 3.13 prints.
        let cases = [
            (115.0, "115.0"),
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (4.702, "4.702"),
            (1e-5, "1e-05"),
            (0.0001, "0.0001"),
            (1e16, "1e+16"),
            (1e15, "1000000000000000.0"),
            (1.5e16, "1.5e+16"),
            (-2.406_593_606_646_068, "-2.406593606646068"),
            (3.617_488_148_663_895e-14, "3.617488148663895e-14"),
            (1e100, "1e+100"),
            (123_456.0, "123456.0"),
            (0.1, "0.1"),
        ];
        for (value, expected) in cases {
            assert_eq!(python_repr(value), expected, "{value:e}");
        }
    }
}
