use std::{fmt, io, marker::PhantomData};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod base;
pub mod bytes;
pub mod traits;

mod imp;

pub use phenix_runtime_macros::{Decodable, Encodable, IsFlag};
pub use traits::{Decodable, Encodable, IsFlag};

pub mod prelude {
    pub use crate::{Decodable, Encodable, IsFlag};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedEof {
    pos: usize,
}

impl UnexpectedEof {
    pub fn new(bytes: &bytes::Bytes<'_>) -> Self {
        Self::new_at(bytes.mark().to_usize())
    }

    pub fn new_at(pos: usize) -> Self {
        Self { pos }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl fmt::Display for UnexpectedEof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unexpected end of input when parsing from byte {}",
            self.pos
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidPrefix {
    pos: usize,
}

impl InvalidPrefix {
    pub fn new(bytes: &bytes::Bytes<'_>) -> Self {
        Self::new_at(bytes.mark().to_usize())
    }

    pub fn new_at(pos: usize) -> Self {
        Self { pos }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl fmt::Display for InvalidPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid prefix at byte {}", self.pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueError {
    pos: usize,
}

impl ValueError {
    pub fn new(bytes: &bytes::Bytes<'_>) -> Self {
        Self::new_at(bytes.mark().to_usize())
    }

    pub fn new_at(pos: usize) -> Self {
        Self { pos }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid value when parsing from byte {}", self.pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecodingError {
    UnexpectedEof(UnexpectedEof),
    InvalidPrefix(InvalidPrefix),
    ValueError(ValueError),
}

impl From<UnexpectedEof> for DecodingError {
    fn from(error: UnexpectedEof) -> Self {
        DecodingError::UnexpectedEof(error)
    }
}

impl From<InvalidPrefix> for DecodingError {
    fn from(error: InvalidPrefix) -> Self {
        DecodingError::InvalidPrefix(error)
    }
}

impl From<ValueError> for DecodingError {
    fn from(error: ValueError) -> Self {
        DecodingError::ValueError(error)
    }
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodingError::UnexpectedEof(error) => fmt::Display::fmt(error, f),
            DecodingError::InvalidPrefix(error) => fmt::Display::fmt(error, f),
            DecodingError::ValueError(error) => fmt::Display::fmt(error, f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Uint(pub u64);

impl From<u64> for Uint {
    fn from(value: u64) -> Self {
        Uint(value)
    }
}

impl From<Uint> for u64 {
    fn from(value: Uint) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Sint(pub i64);

impl From<i64> for Sint {
    fn from(value: i64) -> Self {
        Sint(value)
    }
}

impl From<Sint> for i64 {
    fn from(value: Sint) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Float(pub f64);

impl From<f64> for Float {
    fn from(value: f64) -> Self {
        Float(value)
    }
}

impl From<Float> for f64 {
    fn from(value: Float) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream<T> {
    offset: usize,
    ty: PhantomData<fn() -> T>,
}

impl<T> Stream<T> {
    // Primarily for usage in tests.
    pub fn with_offset(offset: usize) -> Self {
        Self {
            offset,
            ty: PhantomData,
        }
    }
}

impl<T> Default for Stream<T> {
    fn default() -> Self {
        Self {
            offset: 0,
            ty: PhantomData,
        }
    }
}

impl<T: Encodable> Stream<T> {
    pub fn push_encode<W: io::Write>(value: &T, writer: &mut W) -> io::Result<()> {
        value.encode(writer)
    }
}

impl<T: Decodable> Stream<T> {
    pub fn iter<'a>(&self, origin: &'a [u8]) -> StreamIter<'a, T> {
        let mut bytes = bytes::Bytes::new(origin);
        bytes.consume(self.offset);

        StreamIter {
            bytes,
            buf: Vec::new(),
            ty: PhantomData,
        }
    }

    pub fn collect(&self, origin: &[u8]) -> Result<Vec<T>, DecodingError> {
        let mut buf = Vec::new();
        self.iter(origin)
            .map(|value| value?.decode(&mut buf))
            .collect()
    }
}

#[derive(Debug)]
pub struct StreamIter<'a, T> {
    bytes: bytes::Bytes<'a>,
    buf: Vec<u8>,
    ty: PhantomData<fn() -> T>,
}

impl<'a, T: Decodable> Iterator for StreamIter<'a, T> {
    type Item = Result<bytes::ByteSlice<'a, T>, DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        Some(T::recognize(&mut self.bytes, &mut self.buf))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Flags<T> {
    flags: Vec<u8>,
    ty: PhantomData<fn() -> T>,
}

impl<T: IsFlag> Default for Flags<T> {
    fn default() -> Self {
        let flags = if T::IS_EXHAUSTIVE {
            vec![0; Self::n_bytes()]
        } else {
            Vec::new()
        };

        Self {
            flags,
            ty: PhantomData,
        }
    }
}

impl<T: IsFlag> Flags<T> {
    pub fn set(&mut self, flag: T) -> &mut Self {
        if !T::IS_EXHAUSTIVE {
            self.reserve(flag.bit_index());
        }

        base::utils::set_bit_at(flag.bit_index(), &mut self.flags);
        self
    }

    pub fn unset(&mut self, flag: T) -> &mut Self {
        if !T::IS_EXHAUSTIVE {
            self.reserve(flag.bit_index());
        }

        base::utils::clear_bit_at(flag.bit_index(), &mut self.flags);
        self
    }

    pub fn is_set(&self, flag: T) -> bool {
        base::utils::try_test_bit_at(flag.bit_index(), &self.flags).unwrap_or_default()
    }

    pub fn collect(&self) -> Vec<T> {
        T::all()
            .into_iter()
            .filter(|flag| self.is_set(*flag))
            .collect()
    }

    fn n_bytes() -> usize {
        base::bool::byte_size(T::COUNT)
    }

    fn reserve(&mut self, bit: usize) {
        let len = base::utils::byte_index_of(bit) + 1;
        if self.flags.len() < len {
            self.flags.resize(len, 0);
        }
    }
}

impl<T: IsFlag, I: IntoIterator<Item = T>> From<I> for Flags<T> {
    fn from(iter: I) -> Self {
        let mut flags = Self::default();

        for flag in iter {
            flags.set(flag);
        }

        flags
    }
}

impl<T: IsFlag> PartialEq for Flags<T> {
    fn eq(&self, other: &Self) -> bool {
        T::all()
            .into_iter()
            .all(|flag| self.is_set(flag) == other.is_set(flag))
    }
}

impl<T: IsFlag> Eq for Flags<T> {}

impl<T: IsFlag> std::hash::Hash for Flags<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for flag in T::all() {
            self.is_set(flag).hash(state);
        }
    }
}
