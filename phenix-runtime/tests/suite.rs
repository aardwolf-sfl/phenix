use std::{fmt, io::Cursor, marker::PhantomData};

use phenix_runtime::{
    bytes::Bytes, Decodable, Encodable, Flags, Float, InvalidPrefix, IsFlag, Sint, Uint,
    UnexpectedEof, ValueError,
};
use serde::Deserialize;

#[test]
fn encode_uint() {
    TestSuite::<Uint>::run_encode(include_str!("data/uint.json"));
}

#[test]
fn decode_uint() {
    TestSuite::<Uint>::run_decode(include_str!("data/uint.json"));
}

#[test]
fn recognize_uint() {
    TestSuite::<Uint>::run_recognize(include_str!("data/uint.json"));
}

#[test]
fn encode_sint() {
    TestSuite::<Sint>::run_encode(include_str!("data/sint.json"));
}

#[test]
fn decode_sint() {
    TestSuite::<Sint>::run_decode(include_str!("data/sint.json"));
}

#[test]
fn recognize_sint() {
    TestSuite::<Sint>::run_recognize(include_str!("data/sint.json"));
}

#[test]
fn encode_float() {
    TestSuite::<Float>::run_encode(include_str!("data/float.json"));
}

#[test]
fn decode_float() {
    TestSuite::<Float>::run_decode(include_str!("data/float.json"));
}

#[test]
fn recognize_float() {
    TestSuite::<Float>::run_recognize(include_str!("data/float.json"));
}

#[test]
fn encode_bool() {
    TestSuite::<bool>::run_encode(include_str!("data/bool.json"));
}

#[test]
fn decode_bool() {
    TestSuite::<bool>::run_decode(include_str!("data/bool.json"));
}

#[test]
fn recognize_bool() {
    TestSuite::<bool>::run_recognize(include_str!("data/bool.json"));
}

#[test]
fn encode_string() {
    TestSuite::<String>::run_encode(include_str!("data/string.json"));
}

#[test]
fn decode_string() {
    TestSuite::<String>::run_decode(include_str!("data/string.json"));
}

#[test]
fn recognize_string() {
    TestSuite::<String>::run_recognize(include_str!("data/string.json"));
}

#[derive(Debug, PartialEq, Deserialize, Encodable, Decodable)]
struct Struct {
    string: String,
    optional1: Option<Uint>,
    generic: Vec<bool>,
    optional2: Option<Uint>,
}

#[test]
fn encode_struct() {
    TestSuite::<Struct>::run_encode(include_str!("data/struct.json"));
}

#[test]
fn decode_struct() {
    TestSuite::<Struct>::run_decode(include_str!("data/struct.json"));
}

#[test]
fn recognize_struct() {
    TestSuite::<Struct>::run_recognize(include_str!("data/struct.json"));
}

#[derive(Debug, PartialEq, Deserialize, Encodable, Decodable)]
enum Enum {
    Foo,
    Bar { number: Uint, boolean: bool },
}

#[test]
fn encode_enum() {
    TestSuite::<Enum>::run_encode(include_str!("data/enum.json"));
}

#[test]
fn decode_enum() {
    TestSuite::<Enum>::run_decode(include_str!("data/enum.json"));
}

#[test]
fn recognize_enum() {
    TestSuite::<Enum>::run_recognize(include_str!("data/enum.json"));
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, IsFlag)]
enum Flag {
    Foo,
    Bar,
    Baz,
    Qux,
    FooBar,
    FooBaz,
    FooQux,
    BarBaz,
    BarQux,
    BazQux,
}

#[test]
fn encode_flags() {
    TestSuite::<Flags<Flag>, Vec<Flag>>::run_encode(include_str!("data/flags.json"));
}

#[test]
fn decode_flags() {
    TestSuite::<Flags<Flag>, Vec<Flag>>::run_decode(include_str!("data/flags.json"));
}

#[test]
fn recognize_flags() {
    TestSuite::<Flags<Flag>, Vec<Flag>>::run_recognize(include_str!("data/flags.json"));
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, IsFlag)]
#[non_exhaustive]
enum FlagNe {
    Foo,
    Bar,
    Baz,
    Qux,
    FooBar,
    FooBaz,
    FooQux,
    BarBaz,
    BarQux,
    BazQux,
}

#[test]
fn encode_flags_non_exhaustive() {
    TestSuite::<Flags<FlagNe>, Vec<FlagNe>>::run_encode(include_str!("data/flags_ne.json"));
}

#[test]
fn decode_flags_non_exhaustive() {
    TestSuite::<Flags<FlagNe>, Vec<FlagNe>>::run_decode(include_str!("data/flags_ne.json"));
}

#[test]
fn recognize_flags_non_exhaustive() {
    TestSuite::<Flags<FlagNe>, Vec<FlagNe>>::run_recognize(include_str!("data/flags_ne.json"));
}

// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Value<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> Value<T> {
    fn cast<U>(self) -> Value<U>
    where
        T: Into<U>,
    {
        match self {
            Value::One(value) => Value::One(value.into()),
            Value::Many(values) => Value::Many(values.into_iter().map(Into::into).collect()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
#[allow(clippy::enum_variant_names)]
enum Error {
    UnexpectedEof { pos: usize },
    InvalidPrefix { pos: usize },
    ValueError { pos: usize },
}

impl From<Error> for phenix_runtime::DecodingError {
    fn from(error: Error) -> Self {
        match error {
            Error::UnexpectedEof { pos } => UnexpectedEof::new_at(pos).into(),
            Error::InvalidPrefix { pos } => InvalidPrefix::new_at(pos).into(),
            Error::ValueError { pos } => ValueError::new_at(pos).into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TestItem<T> {
    value: Value<T>,
    bytes: Vec<u8>,
    #[serde(default)]
    no_encode: bool,
    #[serde(default)]
    no_decode: bool,
}

#[derive(Debug, Deserialize)]
struct ErrorItem {
    bytes: Vec<u8>,
    #[serde(default)]
    many: Option<usize>,
    error: Error,
    #[serde(default)]
    no_recognize: bool,
}

#[derive(Debug, Deserialize)]
struct TestSuite<T, U = T> {
    tests: Vec<TestItem<U>>,
    #[serde(default)]
    errors: Vec<ErrorItem>,
    #[serde(skip)]
    ty: PhantomData<fn() -> T>,
}

impl<T, U> TestSuite<T, U>
where
    for<'de> U: Deserialize<'de>,
    T: fmt::Debug + PartialEq,
    U: Into<T>,
{
    fn parse(suite: &'static str) -> Self {
        serde_json::from_slice(suite.as_bytes()).unwrap()
    }

    fn run_encode(suite: &'static str)
    where
        T: Encodable,
    {
        let suite = Self::parse(suite);

        for test in suite.tests {
            if test.no_encode {
                continue;
            }

            let mut cursor = Cursor::new(Vec::new());

            match test.value.cast() {
                Value::One(value) => value.encode(&mut cursor).unwrap(),
                Value::Many(values) => T::encode_many(&values, &mut cursor).unwrap(),
            }

            let actual = cursor.into_inner();
            assert_eq!(actual, test.bytes);
        }
    }

    fn run_decode(suite: &'static str)
    where
        T: Decodable,
    {
        let suite = Self::parse(suite);

        for test in suite.tests {
            if test.no_decode {
                continue;
            }

            let bytes = &mut Bytes::new(&test.bytes);
            let buf = &mut Vec::new();

            match test.value.cast() {
                Value::One(expected) => {
                    let value = T::decode(bytes, buf).unwrap();
                    assert_eq!(value, expected);
                }
                Value::Many(expected) => {
                    let mut values = Vec::with_capacity(expected.len());
                    T::decode_many(bytes, buf, expected.len(), &mut values).unwrap();
                    assert_eq!(values, expected);
                }
            }
        }

        for error in suite.errors {
            let bytes = &mut Bytes::new(&error.bytes);
            let buf = &mut Vec::new();

            let actual = match error.many {
                None => T::decode(bytes, buf).unwrap_err(),
                Some(n) => T::decode_many(bytes, buf, n, &mut Vec::new()).unwrap_err(),
            };

            assert_eq!(actual, error.error.into());
        }
    }

    fn run_recognize(suite: &'static str)
    where
        T: Decodable,
    {
        let suite = Self::parse(suite);

        for test in suite.tests {
            if test.no_decode {
                continue;
            }

            let bytes = &mut Bytes::new(&test.bytes);
            let buf = &mut Vec::new();

            match test.value.cast() {
                Value::One(expected) => {
                    let slice = T::recognize(bytes, buf).unwrap();
                    assert_eq!(slice.as_bytes(), &test.bytes);

                    let value = slice.decode(buf).unwrap();
                    assert_eq!(value, expected);
                }
                Value::Many(expected) => {
                    let slice = T::recognize_many(bytes, buf, expected.len()).unwrap();
                    assert_eq!(slice.as_bytes(), &test.bytes);

                    let mut values = Vec::with_capacity(expected.len());
                    slice.decode_many(buf, expected.len(), &mut values).unwrap();
                    assert_eq!(values, expected);
                }
            }
        }

        for error in suite.errors {
            if error.no_recognize {
                continue;
            }

            let bytes = &mut Bytes::new(&error.bytes);
            let buf = &mut Vec::new();

            let actual = match error.many {
                None => T::recognize(bytes, buf).unwrap_err(),
                Some(n) => T::recognize_many(bytes, buf, n).unwrap_err(),
            };

            assert_eq!(actual, error.error.into());
        }
    }
}
