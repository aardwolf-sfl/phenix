use std::{io, marker::PhantomData};

use crate::{
    base,
    bytes::{ByteSlice, Bytes},
    Decodable, DecodingError, Encodable, Flags, IsFlag, UnexpectedEof,
};

impl<T: IsFlag> Encodable for Flags<T> {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        if !T::IS_EXHAUSTIVE {
            base::utils::encode_items_count_relaxed(self.flags.len(), writer)?;
        }

        writer.write_all(self.flags.as_slice())
    }
}

impl<T: IsFlag> Decodable for Flags<T> {
    fn decode(bytes: &mut Bytes<'_>, buf: &mut Vec<u8>) -> Result<Self, DecodingError> {
        let n_bytes = if T::IS_EXHAUSTIVE {
            Self::n_bytes()
        } else {
            base::utils::decode_items_count_relaxed(bytes, buf)?
        };

        if bytes.len() >= n_bytes {
            let flags = bytes[..n_bytes].to_vec();

            Ok(Self {
                flags,
                ty: PhantomData,
            })
        } else {
            Err(UnexpectedEof::new(bytes).into())
        }
    }

    fn recognize<'a>(
        bytes: &mut Bytes<'a>,
        buf: &mut Vec<u8>,
    ) -> Result<ByteSlice<'a, Self>, DecodingError> {
        let mark = bytes.mark();

        let n_bytes = if T::IS_EXHAUSTIVE {
            Self::n_bytes()
        } else {
            base::utils::decode_items_count_relaxed(bytes, buf)?
        };

        if bytes.len() >= n_bytes {
            bytes.consume(n_bytes);
            Ok(bytes.take_slice_from(mark))
        } else {
            Err(UnexpectedEof::new(bytes).into())
        }
    }
}
