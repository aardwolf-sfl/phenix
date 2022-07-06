use std::{io, marker::PhantomData};

use crate::{
    base,
    bytes::{ByteSlice, Bytes},
    Decodable, DecodingError, Encodable, InvalidPrefix, Stream,
};

impl<T: Encodable> Encodable for Vec<T> {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u64;
        base::uint::encode(len, writer)?;

        T::encode_many(self, writer)
    }
}

impl<T: Decodable> Decodable for Vec<T> {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError> {
        let len = base::uint::decode(bytes, buf).map_err(|_| InvalidPrefix::new(bytes))?;
        let len = len as usize;

        let mut values = Vec::with_capacity(len);
        T::decode_many(bytes, buf, len, &mut values)?;

        Ok(values)
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        buf: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        let mark = bytes.mark();

        let len = base::uint::decode(bytes, buf).map_err(|_| InvalidPrefix::new(bytes))?;

        T::recognize_many(bytes, buf, len as usize)?;

        Ok(bytes.take_slice_from(mark))
    }
}

impl<T: Encodable> Encodable for &[T] {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u64;
        base::uint::encode(len, writer)?;

        T::encode_many(self, writer)
    }
}

impl<T> Encodable for Stream<T> {
    fn encode<W: io::Write>(&self, _: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl<T> Decodable for Stream<T> {
    fn decode(bytes: &mut Bytes<'_>, _: &mut Vec<u8>) -> Result<Self, DecodingError> {
        let offset = bytes.mark();
        bytes.consume(bytes.len());

        Ok(Self {
            offset: offset.to_usize(),
            ty: PhantomData,
        })
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        _: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        Ok(bytes.consume_slice(bytes.len()).unwrap())
    }
}
