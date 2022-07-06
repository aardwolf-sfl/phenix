use std::io;

use super::{
    bytes::{ByteSlice, ByteSpan, Bytes},
    DecodingError,
};

pub trait Encodable: Sized {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;

    fn encode_many<W: io::Write>(values: &[Self], writer: &mut W) -> io::Result<()> {
        for item in values {
            item.encode(writer)?;
        }

        Ok(())
    }
}

pub trait Decodable: Sized {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError>;
    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        buf: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError>;

    fn decode_many(
        bytes: &mut Bytes<'_>,
        buf: &mut Vec<u8>,
        n: usize,
        values: &mut Vec<Self>,
    ) -> Result<(), DecodingError> {
        for _ in 0..n {
            values.push(Self::decode(bytes, buf)?);
        }

        Ok(())
    }

    fn recognize_many<'a>(
        bytes: &mut Bytes<'a>,
        buf: &mut Vec<u8>,
        n: usize,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        let mark = bytes.mark();

        for _ in 0..n {
            Self::recognize(bytes, buf)?;
        }

        Ok(bytes.take_slice_from(mark))
    }
}

impl<T: Decodable> ByteSlice<'_, T> {
    pub fn decode(&self, buf: &mut Vec<u8>) -> Result<T, DecodingError> {
        let mut bytes = Bytes::new(self.as_bytes());
        T::decode(&mut bytes, buf)
    }

    pub fn decode_many(
        &self,
        buf: &mut Vec<u8>,
        n: usize,
        values: &mut Vec<T>,
    ) -> Result<(), DecodingError> {
        let mut bytes = Bytes::new(self.as_bytes());
        T::decode_many(&mut bytes, buf, n, values)
    }
}

impl<T: Decodable> ByteSpan<T> {
    pub fn decode(&self, origin: &[u8], buf: &mut Vec<u8>) -> Result<T, DecodingError> {
        let mut bytes = Bytes::new(self.as_bytes(origin));
        T::decode(&mut bytes, buf)
    }

    pub fn decode_many(
        &self,
        origin: &[u8],
        buf: &mut Vec<u8>,
        n: usize,
        values: &mut Vec<T>,
    ) -> Result<(), DecodingError> {
        let mut bytes = Bytes::new(self.as_bytes(origin));
        T::decode_many(&mut bytes, buf, n, values)
    }
}

pub trait IsFlag: Copy {
    type IntoIter: IntoIterator<Item = Self>;

    const COUNT: usize;
    const IS_EXHAUSTIVE: bool;

    fn bit_index(&self) -> usize;
    fn all() -> Self::IntoIter;
}
