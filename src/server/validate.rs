// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Request validation with the upstream server's Pydantic semantics.
//!
//! The upstream request models are Pydantic 2.11 in lax mode, so the API
//! accepts `"115"` for a number, `40.0` or `"40"` for an integer, `true` for
//! `1`, and a zero-time datetime for a date, and reports errors in a fixed
//! shape and field order. This module reproduces the parts of that the
//! request models use, measured against the server in
//! `conformance/expected/http-uk-who.json`.
//!
//! Field order matters because it is the order errors are reported in.
//! Pydantic collects mixin fields in reverse method-resolution order, so a
//! single measurement validates the observation fields before the birth
//! fields, whatever order the JSON uses.
//!
//! Known differences, each a Category 3 fix (`spec/standard.md` section 6.3):
//! a non-string `birth_date` is a 422 here and a bare 500 upstream; `NaN` and
//! lone surrogates, which CPython's JSON parser accepts, are rejected here.

use jiff::civil::Date;
use serde::Serialize;
use serde_json::{Map, Value, json};

/// One part of an error location: a field name or a list index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum LocItem {
    /// A field name.
    Key(String),
    /// A list index.
    Index(usize),
}

impl From<&str> for LocItem {
    fn from(key: &str) -> Self {
        Self::Key(key.to_owned())
    }
}

/// A Pydantic-style validation error, serialised as Pydantic's
/// `errors(include_url=False)` writes it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldError {
    /// Error type, for example `missing`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Where the error is.
    pub loc: Vec<LocItem>,
    /// Human-readable message.
    pub msg: String,
    /// The offending input.
    pub input: Value,
    /// Extra context, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctx: Option<Value>,
}

/// Fields shared by single and bulk requests: `BirthMeasurementsMixin`.
#[derive(Debug, Clone, PartialEq)]
pub struct BirthFields {
    /// `None` only if the request sent an explicit `null`.
    pub gestation_days: Option<i64>,
    /// `None` only if the request sent an explicit `null`.
    pub gestation_weeks: Option<i64>,
    /// `height`, `weight`, `ofc`, or `bmi`.
    pub measurement_method: String,
    /// Date of birth.
    pub birth_date: Date,
    /// `male` or `female`.
    pub sex: String,
}

/// Fields of one observation: `ObservationMeasurementsMixin`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservationFields {
    /// Date of the measurement.
    pub observation_date: Date,
    /// The measurement.
    pub observation_value: f64,
    /// Bone age in years, passed through.
    pub bone_age: Option<f64>,
    /// Bone age method, passed through.
    pub bone_age_type: Option<String>,
    /// Bone age SDS, passed through.
    pub bone_age_sds: Option<f64>,
    /// Bone age centile, passed through.
    pub bone_age_centile: Option<f64>,
    /// Bone age report, passed through.
    pub bone_age_text: Option<String>,
    /// Event labels, passed through unchanged.
    pub events_text: Option<Vec<Value>>,
}

/// `MeasurementRequest`.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequest {
    /// Observation fields.
    pub observation: ObservationFields,
    /// Birth fields.
    pub birth: BirthFields,
}

/// `BulkMeasurementRequest`.
#[derive(Debug, Clone, PartialEq)]
pub struct BulkMeasurementRequest {
    /// Birth fields.
    pub birth: BirthFields,
    /// One or more observations.
    pub observations: Vec<ObservationFields>,
}

const MEASUREMENT_METHODS: &[&str] = &["height", "weight", "ofc", "bmi"];
const SEXES: &[&str] = &["male", "female"];
const BONE_AGE_TYPES: &[&str] = &[
    "greulich-pyle",
    "tanner-whitehouse-ii",
    "tanner-whitehouse-iii",
    "fels",
    "bonexpert",
];

/// Validates a single-measurement body.
///
/// # Errors
///
/// Every field error, in Pydantic's order.
pub fn measurement_request(body: &Value) -> Result<MeasurementRequest, Vec<FieldError>> {
    let mut errors = Vec::new();
    let object = model_object(body, &["body".into()], &mut errors).ok_or(errors.clone())?;
    let mut fields = Fields::new(object, vec!["body".into()], &mut errors);
    let (observation_date, observation) = fields.observation();
    let birth = fields.birth(observation_date);
    match (observation, birth) {
        (Some(observation), Some(birth)) if errors.is_empty() => {
            Ok(MeasurementRequest { observation, birth })
        }
        _ => Err(errors),
    }
}

/// Validates a bulk body.
///
/// # Errors
///
/// Every field error, in Pydantic's order.
pub fn bulk_measurement_request(body: &Value) -> Result<BulkMeasurementRequest, Vec<FieldError>> {
    let mut errors = Vec::new();
    let object = model_object(body, &["body".into()], &mut errors).ok_or(errors.clone())?;
    let mut fields = Fields::new(object, vec!["body".into()], &mut errors);
    let birth = fields.birth(None);
    let observations = fields.observations();
    match (birth, observations) {
        (Some(birth), Some(observations)) if errors.is_empty() => Ok(BulkMeasurementRequest {
            birth,
            observations,
        }),
        _ => Err(errors),
    }
}

fn model_object<'a>(
    value: &'a Value,
    loc: &[LocItem],
    errors: &mut Vec<FieldError>,
) -> Option<&'a Map<String, Value>> {
    if let Value::Object(object) = value {
        return Some(object);
    }
    errors.push(FieldError {
        kind: "model_attributes_type",
        loc: loc.to_vec(),
        msg: "Input should be a valid dictionary or object to extract fields from".to_owned(),
        input: value.clone(),
        ctx: None,
    });
    None
}

/// Validates the fields of one object, accumulating errors.
struct Fields<'a, 'e> {
    object: &'a Map<String, Value>,
    loc: Vec<LocItem>,
    errors: &'e mut Vec<FieldError>,
}

/// A field's value, or that it is absent.
enum Field<'a> {
    Absent,
    Present(&'a Value),
}

impl<'a, 'e> Fields<'a, 'e> {
    fn new(
        object: &'a Map<String, Value>,
        loc: Vec<LocItem>,
        errors: &'e mut Vec<FieldError>,
    ) -> Self {
        Self {
            object,
            loc,
            errors,
        }
    }

    fn get(&self, name: &str) -> Field<'a> {
        self.object.get(name).map_or(Field::Absent, Field::Present)
    }

    fn push(
        &mut self,
        name: &str,
        kind: &'static str,
        msg: impl Into<String>,
        input: &Value,
        ctx: Option<Value>,
    ) {
        let mut loc = self.loc.clone();
        loc.push(name.into());
        self.errors.push(FieldError {
            kind,
            loc,
            msg: msg.into(),
            input: input.clone(),
            ctx,
        });
    }

    fn missing(&mut self, name: &str) {
        let input = Value::Object(self.object.clone());
        self.push(name, "missing", "Field required", &input, None);
    }

    /// A required field; `None` after recording an error.
    fn required<T>(
        &mut self,
        name: &str,
        check: impl FnOnce(&Value) -> Result<T, Problem>,
    ) -> Option<T> {
        match self.get(name) {
            Field::Absent => {
                self.missing(name);
                None
            }
            Field::Present(value) => self.check(name, value, check),
        }
    }

    /// An optional field with a default; explicit `null` gives `None`.
    fn optional<T>(
        &mut self,
        name: &str,
        default: Option<T>,
        check: impl FnOnce(&Value) -> Result<T, Problem>,
    ) -> Option<Option<T>> {
        match self.get(name) {
            Field::Absent => Some(default),
            Field::Present(Value::Null) => Some(None),
            Field::Present(value) => self.check(name, value, check).map(Some),
        }
    }

    fn check<T>(
        &mut self,
        name: &str,
        value: &Value,
        check: impl FnOnce(&Value) -> Result<T, Problem>,
    ) -> Option<T> {
        match check(value) {
            Ok(v) => Some(v),
            Err(problem) => {
                self.push(name, problem.kind, problem.msg, value, problem.ctx);
                None
            }
        }
    }

    /// The observation fields, and the observation date on its own: the
    /// birth-date validator sees it whenever it is valid, even if another
    /// observation field is not.
    fn observation(&mut self) -> (Option<Date>, Option<ObservationFields>) {
        let observation_date = self.required("observation_date", pydantic_date);
        let observation_value = self.required("observation_value", float);
        let bone_age = self.optional("bone_age", None, float);
        let bone_age_type = self.optional("bone_age_type", None, |v| literal(v, BONE_AGE_TYPES));
        let bone_age_sds = self.optional("bone_age_sds", None, float);
        let bone_age_centile = self.optional("bone_age_centile", None, float);
        let bone_age_text = self.optional("bone_age_text", None, string);
        let events_text = self.optional("events_text", None, list);
        let fields = (|| {
            Some(ObservationFields {
                observation_date: observation_date?,
                observation_value: observation_value?,
                bone_age: bone_age?,
                bone_age_type: bone_age_type?,
                bone_age_sds: bone_age_sds?,
                bone_age_centile: bone_age_centile?,
                bone_age_text: bone_age_text?,
                events_text: events_text?,
            })
        })();
        (observation_date, fields)
    }

    fn birth(&mut self, observation_date: Option<Date>) -> Option<BirthFields> {
        let gestation_days = self.optional("gestation_days", Some(0), |v| bounded_int(v, 0, 6));
        let gestation_weeks =
            self.optional("gestation_weeks", Some(40), |v| bounded_int(v, 22, 44));
        let measurement_method =
            self.required("measurement_method", |v| literal(v, MEASUREMENT_METHODS));
        let birth_date = self.required("birth_date", |v| birth_date(v, observation_date));
        let sex = self.required("sex", |v| literal(v, SEXES));
        Some(BirthFields {
            gestation_days: gestation_days?,
            gestation_weeks: gestation_weeks?,
            measurement_method: measurement_method?,
            birth_date: birth_date?,
            sex: sex?,
        })
    }

    fn observations(&mut self) -> Option<Vec<ObservationFields>> {
        let value = match self.get("observations") {
            Field::Absent => {
                self.missing("observations");
                return None;
            }
            Field::Present(value) => value,
        };
        let Value::Array(items) = value else {
            self.push(
                "observations",
                "list_type",
                "Input should be a valid list",
                value,
                None,
            );
            return None;
        };
        let mut observations = Vec::with_capacity(items.len());
        let mut ok = true;
        for (index, item) in items.iter().enumerate() {
            let mut loc = self.loc.clone();
            loc.push("observations".into());
            loc.push(LocItem::Index(index));
            let Some(object) = model_object(item, &loc, self.errors) else {
                ok = false;
                continue;
            };
            let mut fields = Fields::new(object, loc, self.errors);
            match fields.observation().1 {
                Some(observation) => observations.push(observation),
                None => ok = false,
            }
        }
        ok.then_some(observations)
    }
}

/// Why a value failed one field's validation.
#[derive(Debug)]
struct Problem {
    kind: &'static str,
    msg: String,
    ctx: Option<Value>,
}

impl Problem {
    fn new(kind: &'static str, msg: impl Into<String>) -> Self {
        Self {
            kind,
            msg: msg.into(),
            ctx: None,
        }
    }

    fn with_ctx(mut self, ctx: Value) -> Self {
        self.ctx = Some(ctx);
        self
    }
}

/// Pydantic lax `float`.
fn float(value: &Value) -> Result<f64, Problem> {
    let parsing = || {
        Problem::new(
            "float_parsing",
            "Input should be a valid number, unable to parse string as a number",
        )
    };
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| Problem::new("float_type", "Input should be a valid number")),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::String(s) => {
            let trimmed = s.trim();
            let digits_only = trimmed.replace('_', "");
            let underscores_ok =
                !trimmed.starts_with('_') && !trimmed.ends_with('_') && !trimmed.contains("__");
            let parsed: f64 = if underscores_ok {
                digits_only.parse()
            } else {
                trimmed.parse()
            }
            .map_err(|_| parsing())?;
            if parsed.is_nan() {
                // Accepted upstream, then fails with a 500. Rejected here.
                return Err(parsing());
            }
            Ok(parsed)
        }
        _ => Err(Problem::new("float_type", "Input should be a valid number")),
    }
}

/// Pydantic lax `int` with `ge` and `le` constraints.
fn bounded_int(value: &Value, ge: i64, le: i64) -> Result<i64, Problem> {
    let parsing = || {
        Problem::new(
            "int_parsing",
            "Input should be a valid integer, unable to parse string as an integer",
        )
    };
    let n = match value {
        Value::Bool(b) => i64::from(*b),
        Value::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => i,
            (None, Some(f)) if f.fract() != 0.0 => {
                return Err(Problem::new(
                    "int_from_float",
                    "Input should be a valid integer, got a number with a fractional part",
                ));
            }
            #[allow(clippy::cast_possible_truncation, reason = "range checked first")]
            (None, Some(f)) if f.abs() < 9.0e15 => f as i64,
            _ => {
                return Err(Problem::new(
                    "int_parsing_size",
                    "Unable to parse input string as an integer, exceeded maximum size",
                ));
            }
        },
        Value::String(s) => {
            let trimmed = s.trim();
            let (whole, fraction) = trimmed.split_once('.').unwrap_or((trimmed, ""));
            if !fraction.bytes().all(|b| b == b'0') {
                return Err(parsing());
            }
            let unsigned = whole.strip_prefix(['+', '-']).unwrap_or(whole);
            if unsigned.is_empty() || !unsigned.bytes().all(|b| b.is_ascii_digit()) {
                return Err(parsing());
            }
            whole.parse().map_err(|_| {
                Problem::new(
                    "int_parsing_size",
                    "Unable to parse input string as an integer, exceeded maximum size",
                )
            })?
        }
        _ => return Err(Problem::new("int_type", "Input should be a valid integer")),
    };
    if n < ge {
        return Err(Problem::new(
            "greater_than_equal",
            format!("Input should be greater than or equal to {ge}"),
        )
        .with_ctx(json!({ "ge": ge })));
    }
    if n > le {
        return Err(Problem::new(
            "less_than_equal",
            format!("Input should be less than or equal to {le}"),
        )
        .with_ctx(json!({ "le": le })));
    }
    Ok(n)
}

/// Pydantic `Literal[...]` of strings.
fn literal(value: &Value, allowed: &[&str]) -> Result<String, Problem> {
    if let Value::String(s) = value
        && allowed.contains(&s.as_str())
    {
        return Ok(s.clone());
    }
    let quoted: Vec<String> = allowed.iter().map(|a| format!("'{a}'")).collect();
    let expected = match quoted.as_slice() {
        [only] => only.clone(),
        [init @ .., last] => format!("{} or {last}", init.join(", ")),
        [] => String::new(),
    };
    Err(
        Problem::new("literal_error", format!("Input should be {expected}"))
            .with_ctx(json!({ "expected": expected })),
    )
}

fn string(value: &Value) -> Result<String, Problem> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(Problem::new(
            "string_type",
            "Input should be a valid string",
        )),
    }
}

fn list(value: &Value) -> Result<Vec<Value>, Problem> {
    match value {
        Value::Array(items) => Ok(items.clone()),
        _ => Err(Problem::new("list_type", "Input should be a valid list")),
    }
}

/// Wraps a Python `ValueError` raised inside a validator, as Pydantic does.
fn value_error(message: impl std::fmt::Display) -> Problem {
    Problem::new("value_error", format!("Value error, {message}")).with_ctx(json!({ "error": {} }))
}

/// The upstream `birth_date` validators: `datetime.strptime(value,
/// "%Y-%m-%d")`, then a check against the observation date when that is valid.
fn birth_date(value: &Value, observation_date: Option<Date>) -> Result<Date, Problem> {
    let Value::String(text) = value else {
        // Upstream: TypeError escapes the validator and the server returns 500.
        return Err(value_error(format!(
            "strptime() argument 1 must be str, not {}",
            python_type_name(value)
        )));
    };
    let date = strptime_ymd(text).map_err(value_error)?;
    if observation_date.is_some_and(|observed| date > observed) {
        return Err(value_error("Birth date cannot be after observation date."));
    }
    Ok(date)
}

fn python_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "None",
        Value::Bool(_) => "bool",
        Value::Number(n) if n.is_f64() => "float",
        Value::Number(_) => "int",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

/// CPython `datetime.strptime(text, "%Y-%m-%d").date()`, with its messages.
///
/// The format compiles to `(\d\d\d\d)-(1[0-2]|0[1-9]|[1-9])-(3[01]|[12]\d|0[1-9]|[1-9]| [1-9])`,
/// matched at the start with regex alternation order and backtracking.
fn strptime_ymd(text: &str) -> Result<Date, String> {
    let chars: Vec<char> = text.chars().collect();
    let digit = |i: usize| chars.get(i).and_then(|c| c.to_digit(10));
    let no_match = || {
        format!(
            "time data {} does not match format '%Y-%m-%d'",
            python_str_repr(text)
        )
    };

    let year_digits: Option<Vec<u32>> = (0..4).map(digit).collect();
    let year_digits = year_digits.ok_or_else(no_match)?;
    if chars.get(4) != Some(&'-') {
        return Err(no_match());
    }
    let year = year_digits.iter().fold(0, |acc, d| acc * 10 + d);

    // Month alternatives in order; keep the first that is followed by '-'.
    let month_options = [
        (digit(5) == Some(1) && digit(6).is_some_and(|d| d <= 2))
            .then(|| (10 + digit(6).unwrap_or(0), 7)),
        (digit(5) == Some(0) && digit(6).is_some_and(|d| d >= 1))
            .then(|| (digit(6).unwrap_or(0), 7)),
        digit(5).filter(|&d| d >= 1).map(|d| (d, 6)),
    ];
    let (month, after_month) = month_options
        .into_iter()
        .flatten()
        .find(|&(_, end)| chars.get(end) == Some(&'-'))
        .ok_or_else(no_match)?;
    let d = after_month + 1;
    let day_options = [
        (digit(d) == Some(3) && digit(d + 1).is_some_and(|x| x <= 1))
            .then(|| (30 + digit(d + 1).unwrap_or(0), d + 2)),
        (digit(d).is_some_and(|x| x == 1 || x == 2) && digit(d + 1).is_some()).then(|| {
            (
                digit(d).unwrap_or(0) * 10 + digit(d + 1).unwrap_or(0),
                d + 2,
            )
        }),
        (digit(d) == Some(0) && digit(d + 1).is_some_and(|x| x >= 1))
            .then(|| (digit(d + 1).unwrap_or(0), d + 2)),
        digit(d).filter(|&x| x >= 1).map(|x| (x, d + 1)),
        (chars.get(d) == Some(&' ') && digit(d + 1).is_some_and(|x| x >= 1))
            .then(|| (digit(d + 1).unwrap_or(0), d + 2)),
    ];
    let (day, end) = day_options
        .into_iter()
        .flatten()
        .next()
        .ok_or_else(no_match)?;
    if end != chars.len() {
        let rest: String = chars[end..].iter().collect();
        return Err(format!("unconverted data remains: {rest}"));
    }
    if year == 0 {
        return Err("year 0 is out of range".to_owned());
    }
    let year = i16::try_from(year).map_err(|_| "year is out of range".to_owned())?;
    let month = i8::try_from(month).expect("1 to 12");
    let day = i8::try_from(day).expect("1 to 31");
    Date::new(year, month, day).map_err(|_| "day is out of range for month".to_owned())
}

/// CPython `repr(str)` for the characters a date string is likely to hold.
fn python_str_repr(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push(quote);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Pydantic lax `date`, as `speedate` parses it.
fn pydantic_date(value: &Value) -> Result<Date, Problem> {
    let parsing = |error: &str| {
        Problem::new(
            "date_from_datetime_parsing",
            format!("Input should be a valid date or datetime, {error}"),
        )
        .with_ctx(json!({ "error": error }))
    };
    let inexact = || {
        Problem::new(
            "date_from_datetime_inexact",
            "Datetimes provided to dates should have zero time - e.g. be exact dates",
        )
    };
    match value {
        Value::String(text) => {
            let bytes = text.as_bytes();
            if !bytes.is_empty() && bytes.iter().all(u8::is_ascii_digit) {
                let seconds: f64 = text
                    .parse()
                    .map_err(|_| parsing("invalid character in year"))?;
                return from_timestamp(seconds).ok_or_else(inexact);
            }
            parse_iso_date(bytes).map_err(|e| match e {
                DateProblem::Parse(error) => parsing(error),
                DateProblem::Inexact => inexact(),
            })
        }
        Value::Number(n) => n.as_f64().and_then(from_timestamp).ok_or_else(inexact),
        _ => Err(Problem::new("date_type", "Input should be a valid date")),
    }
}

enum DateProblem {
    Parse(&'static str),
    Inexact,
}

/// Unix seconds (or milliseconds above 2e10) to a date, if a whole day.
fn from_timestamp(mut seconds: f64) -> Option<Date> {
    if seconds.abs() > 2e10 {
        seconds /= 1000.0;
    }
    if seconds.fract() != 0.0 || seconds % 86_400.0 != 0.0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, reason = "bounded by the 2e10 check")]
    let days = (seconds / 86_400.0) as i64;
    jiff::civil::date(1970, 1, 1)
        .checked_add(jiff::Span::new().try_days(days).ok()?)
        .ok()
}

fn parse_iso_date(bytes: &[u8]) -> Result<Date, DateProblem> {
    use DateProblem::{Inexact, Parse};
    if bytes.len() < 10 {
        return Err(Parse("input is too short"));
    }
    let number = |range: std::ops::Range<usize>, error| {
        bytes[range]
            .iter()
            .try_fold(0u32, |acc, &b| {
                b.is_ascii_digit().then(|| acc * 10 + u32::from(b - b'0'))
            })
            .ok_or(Parse(error))
    };
    let year = number(0..4, "invalid character in year")?;
    if bytes[4] != b'-' {
        return Err(Parse("invalid date separator, expected `-`"));
    }
    let month = number(5..7, "invalid character in month")?;
    if bytes[7] != b'-' {
        return Err(Parse("invalid date separator, expected `-`"));
    }
    let day = number(8..10, "invalid character in day")?;
    if !(1..=12).contains(&month) {
        return Err(Parse("month value is outside expected range of 1-12"));
    }
    let date = Date::new(
        i16::try_from(year).map_err(|_| Parse("invalid character in year"))?,
        i8::try_from(month).expect("1 to 12"),
        i8::try_from(day).map_err(|_| Parse("day value is outside expected range"))?,
    )
    .map_err(|_| Parse("day value is outside expected range"))?;
    if bytes.len() == 10 {
        return Ok(date);
    }
    if !matches!(bytes[10], b'T' | b't' | b'_' | b' ') {
        return Err(Parse(
            "invalid datetime separator, expected `T`, `t`, `_` or space",
        ));
    }
    if zero_time(&bytes[11..])? {
        Ok(date)
    } else {
        Err(Inexact)
    }
}

/// Parses `HH:MM[:SS[.frac]][Z|±HH[:]MM]`; `true` if the time is midnight.
fn zero_time(bytes: &[u8]) -> Result<bool, DateProblem> {
    use DateProblem::Parse;
    let two = |i: usize, error| match (bytes.get(i), bytes.get(i + 1)) {
        (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit() => {
            Ok(u32::from(a - b'0') * 10 + u32::from(b - b'0'))
        }
        (None, _) | (_, None) => Err(Parse("input is too short")),
        _ => Err(Parse(error)),
    };
    let hour = two(0, "invalid character in hour")?;
    if bytes.get(2) != Some(&b':') {
        return Err(Parse("invalid time separator, expected `:`"));
    }
    let minute = two(3, "invalid character in minute")?;
    let mut i = 5;
    let mut second = 0;
    let mut fraction_zero = true;
    if bytes.get(i) == Some(&b':') {
        second = two(i + 1, "invalid character in second")?;
        i += 3;
        if matches!(bytes.get(i), Some(b'.' | b',')) {
            i += 1;
            let start = i;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                fraction_zero &= bytes[i] == b'0';
                i += 1;
            }
            if i == start {
                return Err(Parse("second fraction value is more than 6 digits long"));
            }
        }
    }
    if hour > 23 {
        return Err(Parse("hour value is outside expected range of 0-23"));
    }
    if minute > 59 {
        return Err(Parse("minute value is outside expected range of 0-59"));
    }
    if second > 59 {
        return Err(Parse("second value is outside expected range of 0-59"));
    }
    match &bytes[i..] {
        [] | [b'Z' | b'z'] => {}
        [b'+' | b'-', rest @ ..] if matches!(rest.len(), 2 | 4 | 5) => {}
        _ => return Err(Parse("invalid timezone sign")),
    }
    Ok(hour == 0 && minute == 0 && second == 0 && fraction_zero)
}

/// What `centile_format` asked for.
#[derive(Debug, Clone, PartialEq)]
pub enum CentileFormatRequest {
    /// One of the literal names, which upstream serves from stored files.
    Named(String),
    /// A list of centiles, or SDS when `is_sds` is true.
    Custom(Vec<f64>),
    /// An explicit `null`, which upstream accepts and then fails to find.
    Null,
}

/// `ChartCoordinateRequest`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartCoordinateRequest {
    /// `male` or `female`.
    pub sex: String,
    /// `height`, `weight`, `ofc`, or `bmi`.
    pub measurement_method: String,
    /// Whether a custom list is of SDS rather than centiles.
    pub is_sds: bool,
    /// The centile collection.
    pub centile_format: CentileFormatRequest,
}

const CENTILE_FORMATS: &[&str] = &[
    "cole-nine-centiles",
    "extended-who-centiles",
    "three-percent-centiles",
    "five-percent-centiles",
    "eighty-five-percent-centiles",
];

/// Validates a chart-coordinates body, including the upstream model
/// validator's limits on custom lists.
///
/// # Errors
///
/// Every field error, in Pydantic's order; or the model validator's error.
pub fn chart_coordinate_request(body: &Value) -> Result<ChartCoordinateRequest, Vec<FieldError>> {
    let mut errors = Vec::new();
    let object = model_object(body, &["body".into()], &mut errors).ok_or(errors.clone())?;
    let mut fields = Fields::new(object, vec!["body".into()], &mut errors);
    let sex = fields.required("sex", |v| literal(v, SEXES));
    let measurement_method =
        fields.required("measurement_method", |v| literal(v, MEASUREMENT_METHODS));
    let is_sds = fields.flag("is_sds");
    let centile_format = fields.centile_format();
    let (Some(sex), Some(measurement_method), Some(is_sds), Some(centile_format)) =
        (sex, measurement_method, is_sds, centile_format)
    else {
        return Err(errors);
    };
    if !errors.is_empty() {
        return Err(errors);
    }

    // The upstream `custom_centiles_must_not_exceed_fifteen` model validator.
    if let CentileFormatRequest::Custom(values) = &centile_format {
        let message = if values.len() > 15 {
            Some("Centile/SDS formats cannot exceed 15 items.")
        } else if values.is_empty() {
            Some(
                "Empty list. Please provide at least one value or one of the standard collection flags.",
            )
        } else if !is_sds && values.iter().any(|&c| c < 0.0) {
            Some("Centile values cannot be negative.")
        } else {
            None
        };
        if let Some(message) = message {
            let problem = value_error(message);
            return Err(vec![FieldError {
                kind: problem.kind,
                loc: vec!["body".into()],
                msg: problem.msg,
                input: body.clone(),
                ctx: problem.ctx,
            }]);
        }
    }
    Ok(ChartCoordinateRequest {
        sex,
        measurement_method,
        is_sds,
        centile_format,
    })
}

/// `FictionalChildRequest`. Every optional field may be an explicit `null`,
/// which upstream accepts and then trips over, so each is kept as an `Option`
/// for the route to reproduce what happens next.
#[derive(Debug, Clone, PartialEq)]
pub struct FictionalChildRequest {
    /// `height`, `weight`, `ofc`, or `bmi`.
    pub measurement_method: String,
    /// `male` or `female`.
    pub sex: String,
    /// Decimal years; default 0.
    pub start_chronological_age: Option<f64>,
    /// Decimal years; default 20.
    pub end_age: Option<f64>,
    /// Default 40.
    pub gestation_weeks: Option<i64>,
    /// Default 0.
    pub gestation_days: Option<i64>,
    /// One of upstream's unit names; default `months`.
    pub measurement_interval_type: Option<String>,
    /// Default 20.
    pub measurement_interval_number: Option<i64>,
    /// Default 0.
    pub start_sds: Option<f64>,
    /// Whether the SDS drifts.
    pub drift: bool,
    /// Default -0.05.
    pub drift_range: Option<f64>,
    /// Whether to add measurement noise.
    pub noise: bool,
    /// Default 0.005.
    pub noise_range: Option<f64>,
}

const INTERVAL_TYPES: &[&str] = &[
    "d", "day", "days", "w", "week", "weeks", "m", "month", "months", "y", "year", "years",
];

/// Validates a fictional-child body.
///
/// # Errors
///
/// Every field error, in Pydantic's order.
pub fn fictional_child_request(body: &Value) -> Result<FictionalChildRequest, Vec<FieldError>> {
    let mut errors = Vec::new();
    let object = model_object(body, &["body".into()], &mut errors).ok_or(errors.clone())?;
    let mut fields = Fields::new(object, vec!["body".into()], &mut errors);
    let measurement_method =
        fields.required("measurement_method", |v| literal(v, MEASUREMENT_METHODS));
    let sex = fields.required("sex", |v| literal(v, SEXES));
    let start_chronological_age = fields.optional("start_chronological_age", Some(0.0), float);
    let end_age = fields.optional("end_age", Some(20.0), float);
    let gestation_weeks = fields.optional("gestation_weeks", Some(40), |v| bounded_int(v, 22, 44));
    let gestation_days = fields.optional("gestation_days", Some(0), |v| bounded_int(v, 0, 6));
    let measurement_interval_type = fields.optional(
        "measurement_interval_type",
        Some("months".to_owned()),
        |v| literal(v, INTERVAL_TYPES),
    );
    let measurement_interval_number =
        fields.optional("measurement_interval_number", Some(20), |v| {
            bounded_int(v, i64::MIN, i64::MAX)
        });
    let start_sds = fields.optional("start_sds", Some(0.0), float);
    let drift = fields.flag("drift");
    let drift_range = fields.optional("drift_range", Some(-0.05), float);
    let noise = fields.flag("noise");
    let noise_range = fields.optional("noise_range", Some(0.005), float);
    let request = (|| {
        Some(FictionalChildRequest {
            measurement_method: measurement_method?,
            sex: sex?,
            start_chronological_age: start_chronological_age?,
            end_age: end_age?,
            gestation_weeks: gestation_weeks?,
            gestation_days: gestation_days?,
            measurement_interval_type: measurement_interval_type?,
            measurement_interval_number: measurement_interval_number?,
            start_sds: start_sds?,
            drift: drift?,
            drift_range: drift_range?,
            noise: noise?,
            noise_range: noise_range?,
        })
    })();
    match request {
        Some(request) if errors.is_empty() => Ok(request),
        _ => Err(errors),
    }
}

/// `MidParentalHeightRequest`.
#[derive(Debug, Clone, PartialEq)]
pub struct MidParentalHeightRequest {
    /// The father's height in cm.
    pub height_paternal: f64,
    /// The mother's height in cm.
    pub height_maternal: f64,
    /// The child's sex: `male` or `female`.
    pub sex: String,
    /// `uk-who` (the default), `cdc`, or `who`.
    pub reference: String,
}

const MID_PARENTAL_HEIGHT_REFERENCES: &[&str] = &["uk-who", "cdc", "who"];

/// Validates a mid-parental height body.
///
/// # Errors
///
/// Every field error, in Pydantic's order.
pub fn mid_parental_height_request(
    body: &Value,
) -> Result<MidParentalHeightRequest, Vec<FieldError>> {
    let mut errors = Vec::new();
    let object = model_object(body, &["body".into()], &mut errors).ok_or(errors.clone())?;
    let mut fields = Fields::new(object, vec!["body".into()], &mut errors);
    let height_paternal = fields.required("height_paternal", float);
    let height_maternal = fields.required("height_maternal", float);
    let sex = fields.required("sex", |v| literal(v, SEXES));
    let reference = match fields.get("reference") {
        Field::Absent => Some("uk-who".to_owned()),
        Field::Present(value) => fields.check("reference", value, |v| {
            literal(v, MID_PARENTAL_HEIGHT_REFERENCES)
        }),
    };
    let request = (|| {
        Some(MidParentalHeightRequest {
            height_paternal: height_paternal?,
            height_maternal: height_maternal?,
            sex: sex?,
            reference: reference?,
        })
    })();
    match request {
        Some(request) if errors.is_empty() => Ok(request),
        _ => Err(errors),
    }
}

impl Fields<'_, '_> {
    /// A `bool` field defaulting to `False`, which may not be `null`.
    fn flag(&mut self, name: &str) -> Option<bool> {
        match self.get(name) {
            Field::Absent => Some(false),
            Field::Present(value) => self.check(name, value, boolean),
        }
    }

    /// `Optional[Union[Literal[...], List[float]]]`, defaulting to the Cole
    /// nine centiles. When both union members fail, Pydantic reports each,
    /// tagged with the member's name in the location.
    fn centile_format(&mut self) -> Option<CentileFormatRequest> {
        let value = match self.get("centile_format") {
            Field::Absent => {
                return Some(CentileFormatRequest::Named("cole-nine-centiles".to_owned()));
            }
            Field::Present(Value::Null) => return Some(CentileFormatRequest::Null),
            Field::Present(value) => value,
        };
        let literal_result = literal(value, CENTILE_FORMATS);
        if let Ok(name) = literal_result {
            return Some(CentileFormatRequest::Named(name));
        }
        let list_problems: Vec<(Option<usize>, Problem, Value)> = match value {
            Value::Array(items) => {
                let mut values = Vec::with_capacity(items.len());
                let mut problems = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    match float(item) {
                        Ok(v) => values.push(v),
                        Err(problem) => problems.push((Some(index), problem, item.clone())),
                    }
                }
                if problems.is_empty() {
                    return Some(CentileFormatRequest::Custom(values));
                }
                problems
            }
            other => vec![(
                None,
                Problem::new("list_type", "Input should be a valid list"),
                other.clone(),
            )],
        };

        let base = {
            let mut loc = self.loc.clone();
            loc.push("centile_format".into());
            loc
        };
        let literal_tag = format!(
            "literal[{}]",
            CENTILE_FORMATS
                .iter()
                .map(|f| format!("'{f}'"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let problem = literal_result.expect_err("checked above");
        let mut loc = base.clone();
        loc.push(LocItem::Key(literal_tag));
        self.errors.push(FieldError {
            kind: problem.kind,
            loc,
            msg: problem.msg,
            input: value.clone(),
            ctx: problem.ctx,
        });
        for (index, problem, input) in list_problems {
            let mut loc = base.clone();
            loc.push("list[float]".into());
            if let Some(index) = index {
                loc.push(LocItem::Index(index));
            }
            self.errors.push(FieldError {
                kind: problem.kind,
                loc,
                msg: problem.msg,
                input,
                ctx: problem.ctx,
            });
        }
        None
    }
}

/// Pydantic lax `bool`.
fn boolean(value: &Value) -> Result<bool, Problem> {
    let parsing = || {
        Problem::new(
            "bool_parsing",
            "Input should be a valid boolean, unable to interpret input",
        )
    };
    match value {
        Value::Bool(b) => Ok(*b),
        Value::Number(n) => {
            let v = n.as_f64().unwrap_or(f64::NAN);
            if v == 0.0 {
                Ok(false)
            } else if v == 1.0 {
                Ok(true)
            } else {
                Err(parsing())
            }
        }
        Value::String(s) => match s.to_ascii_lowercase().as_str() {
            "0" | "off" | "f" | "false" | "n" | "no" => Ok(false),
            "1" | "on" | "t" | "true" | "y" | "yes" => Ok(true),
            _ => Err(parsing()),
        },
        _ => Err(Problem::new("bool_type", "Input should be a valid boolean")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strptime_matches_cpython() {
        let ok = |t| strptime_ymd(t).unwrap().to_string();
        assert_eq!(ok("2020-4-1"), "2020-04-01");
        assert_eq!(ok("2020-04- 1"), "2020-04-01");
        assert_eq!(
            strptime_ymd("2020-04-1 ").unwrap_err(),
            "unconverted data remains:  "
        );
        assert_eq!(
            strptime_ymd("2020-02-30").unwrap_err(),
            "day is out of range for month"
        );
        assert_eq!(
            strptime_ymd("2020-13-01").unwrap_err(),
            "time data '2020-13-01' does not match format '%Y-%m-%d'"
        );
        assert_eq!(
            strptime_ymd("2020-04-123").unwrap_err(),
            "unconverted data remains: 3"
        );
        assert_eq!(
            strptime_ymd("2020-04-12T00:00:00").unwrap_err(),
            "unconverted data remains: T00:00:00"
        );
    }

    #[test]
    fn lax_numbers_match_pydantic() {
        assert_eq!(bounded_int(&json!("40.0"), 22, 44).unwrap(), 40);
        assert_eq!(bounded_int(&json!(" 40 "), 22, 44).unwrap(), 40);
        assert_eq!(
            bounded_int(&json!("4e1"), 22, 44).unwrap_err().kind,
            "int_parsing"
        );
        assert_eq!(
            bounded_int(&json!(40.5), 22, 44).unwrap_err().kind,
            "int_from_float"
        );
        assert_eq!(
            bounded_int(&json!(true), 22, 44).unwrap_err().kind,
            "greater_than_equal"
        );
        assert_eq!(float(&json!(" 1_000 ")).unwrap(), 1000.0);
        assert_eq!(float(&json!(".5")).unwrap(), 0.5);
        assert_eq!(float(&json!("0x10")).unwrap_err().kind, "float_parsing");
    }

    #[test]
    fn dates_match_speedate() {
        let kind = |v: Value| {
            pydantic_date(&v)
                .map(|d| d.to_string())
                .map_err(|p| (p.kind, p.msg))
        };
        assert_eq!(kind(json!("2028-06-12 00:00")).unwrap(), "2028-06-12");
        assert_eq!(kind(json!("2028-06-12T00:00:00Z")).unwrap(), "2028-06-12");
        assert_eq!(kind(json!(0)).unwrap(), "1970-01-01");
        assert_eq!(
            kind(json!("2028-06-12T10:00:00")).unwrap_err().0,
            "date_from_datetime_inexact"
        );
        assert_eq!(
            kind(json!("20280612")).unwrap_err().0,
            "date_from_datetime_inexact"
        );
        assert!(
            kind(json!("2028-02-30"))
                .unwrap_err()
                .1
                .ends_with("day value is outside expected range")
        );
        assert!(
            kind(json!("2028/06/12"))
                .unwrap_err()
                .1
                .ends_with("invalid date separator, expected `-`")
        );
        assert!(
            kind(json!("xxxxxxxxxxxx"))
                .unwrap_err()
                .1
                .ends_with("invalid character in year")
        );
    }
}
