use std::{io, mem};

use crate::{
    base,
    bytes::{ByteSlice, Bytes},
    Decodable, DecodingError, Encodable, Float, InvalidPrefix, Sint, Uint, UnexpectedEof,
    ValueError,
};

macro_rules! impl_num {
    ($num:ty) => {
        impl Encodable for $num {
            fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
                writer.write_all(&self.to_le_bytes())
            }
        }

        impl Decodable for $num {
            fn decode(bytes: &mut Bytes<'_>, _: &mut Vec<u8>) -> Result<Self, DecodingError> {
                bytes
                    .consume_bytes(mem::size_of::<$num>())
                    .map(|bytes| <$num>::from_le_bytes(bytes.try_into().unwrap()))
                    .ok_or_else(|| UnexpectedEof::new(bytes).into())
            }

            fn recognize<'a>(
                bytes: &mut Bytes<'a>,
                _: &mut Vec<u8>,
            ) -> Result<ByteSlice<'a, Self>, DecodingError> {
                bytes
                    .consume_slice(mem::size_of::<$num>())
                    .ok_or_else(|| UnexpectedEof::new(bytes).into())
            }

            fn recognize_many<'a>(
                bytes: &mut Bytes<'a>,
                _: &mut Vec<u8>,
                n: usize,
            ) -> Result<ByteSlice<'a, Self>, DecodingError> {
                bytes
                    .consume_slice(n * mem::size_of::<$num>())
                    .ok_or_else(|| UnexpectedEof::new(bytes).into())
            }
        }
    };
}

impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);

impl_num!(i8);
impl_num!(i16);
impl_num!(i32);
impl_num!(i64);

impl_num!(f32);
impl_num!(f64);

impl Encodable for bool {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        base::bool::encode(*self, writer)
    }

    fn encode_many<W: io::Write>(values: &[Self], writer: &mut W) -> io::Result<()> {
        base::bool::encode_many(values, writer)
    }
}

impl Decodable for bool {
    fn decode(bytes: &mut Bytes<'_>, _: &mut Vec<u8>) -> Result<Self, DecodingError> {
        base::bool::decode(bytes)
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        base::bool::recognize(bytes)
    }

    fn decode_many<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
        n: usize,
        values: &mut Vec<Self>,
    ) -> Result<(), DecodingError> {
        base::bool::decode_many(bytes, n, values)
    }

    fn recognize_many<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
        n: usize,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        base::bool::recognize_many(bytes, n)
    }
}

impl Encodable for String {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u64;
        base::uint::encode(len, writer)?;

        writer.write_all(self.as_bytes())
    }
}

impl Decodable for String {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError> {
        let len = base::uint::decode(bytes, buf).map_err(|_| InvalidPrefix::new(bytes))?;
        let mark = bytes.mark();

        let bytes = bytes
            .consume_bytes(len as usize)
            .ok_or_else(|| UnexpectedEof::new(bytes))?;

        match std::str::from_utf8(bytes) {
            Ok(string) => Ok(string.to_string()),
            Err(error) => Err(ValueError::new_at(mark.to_usize() + error.valid_up_to()).into()),
        }
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        buf: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        let mark = bytes.mark();

        let len = base::uint::decode(bytes, buf).map_err(|_| InvalidPrefix::new(bytes))?;
        let len = len as usize;

        if bytes.len() < len {
            return Err(UnexpectedEof::new(bytes).into());
        }

        bytes.consume(len);
        Ok(bytes.take_slice_from(mark))
    }
}

impl Encodable for &str {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u64;
        base::uint::encode(len, writer)?;

        writer.write_all(self.as_bytes())
    }
}

impl Encodable for Uint {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        base::uint::encode(self.0, writer)
    }
}

impl Decodable for Uint {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError> {
        base::uint::decode(bytes, buf).map(Uint)
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        base::uint::recognize(bytes).map(ByteSlice::cast)
    }
}

impl Encodable for Sint {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        base::sint::encode(self.0, writer)
    }
}

impl Decodable for Sint {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError> {
        base::sint::decode(bytes, buf).map(Sint)
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        base::sint::recognize(bytes).map(ByteSlice::cast)
    }
}

impl Encodable for Float {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        base::float::encode(self.0, writer)
    }
}

impl Decodable for Float {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError> {
        base::float::decode(bytes, buf).map(Float)
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        base::float::recognize(bytes).map(ByteSlice::cast)
    }
}
