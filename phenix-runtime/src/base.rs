use std::{io, mem};

use super::{
    bytes::{ByteSlice, Bytes},
    DecodingError, UnexpectedEof,
};

pub mod uint {
    // https://capnproto.org/encoding.html#packing
    // + extension:
    //   * if 0 <= n <= (255 - 8) -> n itself is the first byte
    //   * otherwise (255 - 8) + (bytes needed for n) is the first byte

    use super::*;

    const N_BYTES_SHIFT: u8 = u8::MAX - mem::size_of::<u64>() as u8;

    pub fn encode<W: io::Write>(value: u64, writer: &mut W) -> io::Result<()> {
        let bytes = value.to_le_bytes();

        let n_bytes = if value <= N_BYTES_SHIFT as u64 {
            return writer.write_all(&[value as u8]);
        } else if bytes.last().copied().unwrap() == 0 {
            bytes
                .iter()
                .copied()
                .enumerate()
                .rev()
                .find_map(|(i, b)| (b != 0).then(|| i + 1))
                .unwrap_or(1)
        } else {
            bytes.len()
        };

        writer.write_all(&[n_bytes as u8 + N_BYTES_SHIFT])?;
        writer.write_all(&bytes[..n_bytes])
    }

    pub fn decode(bytes: &mut Bytes<'_>) -> Result<u64, DecodingError> {
        let mut buf = [0u8; mem::size_of::<u64>()];

        let small = bytes
            .first()
            .copied()
            .ok_or_else(|| UnexpectedEof::new(bytes))?;

        if small <= N_BYTES_SHIFT {
            bytes.consume(1);
            return Ok(small as u64);
        }

        let n_bytes = (small - N_BYTES_SHIFT) as usize;

        if bytes.len() < 1 + n_bytes {
            return Err(UnexpectedEof::new(bytes).into());
        }

        bytes.consume(1);
        buf[..n_bytes].copy_from_slice(bytes.consume_bytes(n_bytes).unwrap());

        Ok(u64::from_le_bytes(buf.as_slice().try_into().unwrap()))
    }

    pub fn recognize<'a>(bytes: &mut Bytes<'a>) -> Result<ByteSlice<'a, u64>, DecodingError> {
        let small = bytes
            .first()
            .copied()
            .ok_or_else(|| UnexpectedEof::new(bytes))?;

        if small <= N_BYTES_SHIFT {
            return Ok(bytes.consume_slice(1).unwrap());
        }

        let n_bytes = (small - N_BYTES_SHIFT) as usize;

        bytes
            .consume_slice(1 + n_bytes)
            .ok_or_else(|| UnexpectedEof::new(bytes).into())
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use proptest::prelude::*;

        use super::*;

        fn encode_to_bytes(value: u64) -> Vec<u8> {
            let mut cursor = Cursor::new(Vec::new());
            encode(value, &mut cursor).unwrap();
            cursor.into_inner()
        }

        fn decode_from_bytes(value: &[u8]) -> Result<u64, DecodingError> {
            let mut bytes = Bytes::new(value);
            decode(&mut bytes)
        }

        fn recognize_from_bytes(value: &[u8]) -> Result<ByteSlice<'_, u64>, DecodingError> {
            let mut bytes = Bytes::new(value);
            recognize(&mut bytes)
        }

        fn decode_from_byte_slice(slice: ByteSlice<'_, u64>) -> Result<u64, DecodingError> {
            decode_from_bytes(slice.as_bytes())
        }

        proptest! {
            #[test]
            fn roundtrip(value: u64) {
                let bytes = encode_to_bytes(value);

                assert_eq!(decode_from_bytes(&bytes), Ok(value));
                assert_eq!(
                    recognize_from_bytes(&bytes).and_then(decode_from_byte_slice),
                    Ok(value)
                );
            }

            #[test]
            fn fuzz(bytes: Vec<u8>) {
                let decode_result = decode_from_bytes(&bytes);
                let recognize_result = recognize_from_bytes(&bytes);
                assert_eq!(decode_result.is_ok(), recognize_result.is_ok());
            }
        }
    }
}

pub mod sint {
    // https://developers.google.com/protocol-buffers/docs/encoding#signed-ints

    use super::*;

    pub fn encode<W: io::Write>(value: i64, writer: &mut W) -> io::Result<()> {
        let value = value >> (i64::BITS - 1) ^ (value << 1);
        let value = u64::from_le_bytes(value.to_le_bytes());

        super::uint::encode(value, writer)
    }

    pub fn decode(bytes: &mut Bytes<'_>) -> Result<i64, DecodingError> {
        let value = super::uint::decode(bytes)?;

        let value = (value >> 1) ^ (!(value & 1)).wrapping_add(1);
        Ok(i64::from_le_bytes(value.to_le_bytes()))
    }

    pub fn recognize<'a>(bytes: &mut Bytes<'a>) -> Result<ByteSlice<'a, i64>, DecodingError> {
        super::uint::recognize(bytes).map(ByteSlice::cast)
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use proptest::prelude::*;

        use super::*;

        fn encode_to_bytes(value: i64) -> Vec<u8> {
            let mut cursor = Cursor::new(Vec::new());
            encode(value, &mut cursor).unwrap();
            cursor.into_inner()
        }

        fn decode_from_bytes(value: &[u8]) -> Result<i64, DecodingError> {
            let mut bytes = Bytes::new(value);
            decode(&mut bytes)
        }

        fn recognize_from_bytes(value: &[u8]) -> Result<ByteSlice<'_, i64>, DecodingError> {
            let mut bytes = Bytes::new(value);
            recognize(&mut bytes)
        }

        fn decode_from_byte_slice(slice: ByteSlice<'_, i64>) -> Result<i64, DecodingError> {
            decode_from_bytes(slice.as_bytes())
        }

        proptest! {
            #[test]
            fn roundtrip(value: i64) {
                let bytes = encode_to_bytes(value);

                assert_eq!(decode_from_bytes(&bytes), Ok(value));
                assert_eq!(
                    recognize_from_bytes(&bytes).and_then(decode_from_byte_slice),
                    Ok(value)
                );
            }

            #[test]
            fn fuzz(bytes: Vec<u8>) {
                let decode_result = decode_from_bytes(&bytes);
                let recognize_result = recognize_from_bytes(&bytes);
                assert_eq!(decode_result.is_ok(), recognize_result.is_ok());
            }
        }
    }
}

pub mod float {
    // encoding bytes of float in big-endian as uint in little endian - "nice"
    // floats have long tail of zeros and the endianess switching trick then
    // results into a relatively small integer that can be encoded into a
    // compact form.

    use super::*;

    pub fn encode<W: io::Write>(value: f64, writer: &mut W) -> io::Result<()> {
        let value = u64::from_le_bytes(value.to_be_bytes());
        super::uint::encode(value, writer)
    }

    pub fn decode(bytes: &mut Bytes<'_>) -> Result<f64, DecodingError> {
        let value = super::uint::decode(bytes)?;
        Ok(f64::from_be_bytes(value.to_le_bytes()))
    }

    pub fn recognize<'a>(bytes: &mut Bytes<'a>) -> Result<ByteSlice<'a, f64>, DecodingError> {
        super::uint::recognize(bytes).map(ByteSlice::cast)
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use proptest::prelude::*;

        use super::*;

        fn encode_to_bytes(value: f64) -> Vec<u8> {
            let mut cursor = Cursor::new(Vec::new());
            encode(value, &mut cursor).unwrap();
            cursor.into_inner()
        }

        fn decode_from_bytes(value: &[u8]) -> Result<f64, DecodingError> {
            let mut bytes = Bytes::new(value);
            decode(&mut bytes)
        }

        fn recognize_from_bytes(value: &[u8]) -> Result<ByteSlice<'_, f64>, DecodingError> {
            let mut bytes = Bytes::new(value);
            recognize(&mut bytes)
        }

        fn decode_from_byte_slice(slice: ByteSlice<'_, f64>) -> Result<f64, DecodingError> {
            decode_from_bytes(slice.as_bytes())
        }

        #[test]
        fn special_values() {
            assert_eq!(encode_to_bytes(f64::NAN), vec![249, 127, 248]);
            assert_eq!(encode_to_bytes(f64::INFINITY), vec![249, 127, 240]);
            assert_eq!(encode_to_bytes(f64::NEG_INFINITY), vec![249, 255, 240]);

            assert!(decode_from_bytes(&[249, 127, 248]).unwrap().is_nan());
            assert_eq!(decode_from_bytes(&[249, 127, 240]).unwrap(), f64::INFINITY);
            assert_eq!(
                decode_from_bytes(&[249, 255, 240]).unwrap(),
                f64::NEG_INFINITY
            );
        }

        proptest! {
            #[test]
            fn roundtrip(value: f64) {
                let bytes = encode_to_bytes(value);

                assert_eq!(decode_from_bytes(&bytes), Ok(value));
                assert_eq!(
                    recognize_from_bytes(&bytes).and_then(decode_from_byte_slice),
                    Ok(value)
                );
            }

            #[test]
            fn fuzz(bytes: Vec<u8>) {
                let decode_result = decode_from_bytes(&bytes);
                let recognize_result = recognize_from_bytes(&bytes);
                assert_eq!(decode_result.is_ok(), recognize_result.is_ok());
            }
        }
    }
}

pub mod bool {
    use super::*;

    pub fn encode<W: io::Write>(value: bool, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[value as u8])
    }

    pub fn encode_many<W: io::Write>(values: &[bool], writer: &mut W) -> io::Result<()> {
        for chunk in values.chunks(u8::BITS as usize) {
            let mut byte = 0;

            for b in chunk.iter().copied().rev() {
                byte <<= 1;

                if b {
                    byte |= 1;
                }
            }

            writer.write_all(&[byte])?;
        }

        Ok(())
    }

    pub fn decode(bytes: &mut Bytes<'_>) -> Result<bool, DecodingError> {
        if bytes.len() > 0 {
            let value = bytes[0] & 0x01 != 0;
            bytes.consume(1);

            Ok(value)
        } else {
            Err(UnexpectedEof::new(bytes).into())
        }
    }

    pub fn decode_many(
        bytes: &mut Bytes<'_>,
        n: usize,
        values: &mut Vec<bool>,
    ) -> Result<(), DecodingError> {
        if n == 0 {
            return Ok(());
        }

        let (n_bytes, div, rem) = byte_size_extra(n);

        let bytes = bytes
            .consume_bytes(n_bytes)
            .ok_or_else(|| UnexpectedEof::new(bytes))?;

        // Iterate over bytes, but ignore the last byte if it is "incomplete".
        for mut byte in bytes.iter().copied().take(div) {
            for _ in 0..u8::BITS {
                values.push(byte & 0x01 != 0);
                byte >>= 1;
            }
        }

        if rem > 0 {
            // Process the incomplete byte.
            let mut byte = bytes[div];

            for _ in 0..rem {
                values.push(byte & 0x01 != 0);
                byte >>= 1;
            }
        }

        Ok(())
    }

    pub fn recognize<'a>(bytes: &mut Bytes<'a>) -> Result<ByteSlice<'a, bool>, DecodingError> {
        bytes
            .consume_slice(1)
            .ok_or_else(|| UnexpectedEof::new(bytes).into())
    }

    pub fn recognize_many<'a>(
        bytes: &mut Bytes<'a>,
        n: usize,
    ) -> Result<ByteSlice<'a, bool>, DecodingError> {
        let n_bytes = byte_size(n);

        bytes
            .consume_slice(n_bytes)
            .ok_or_else(|| UnexpectedEof::new(bytes).into())
    }

    pub fn byte_size_extra(n_bits: usize) -> (usize, usize, usize) {
        let div = n_bits / u8::BITS as usize;
        let rem = n_bits % u8::BITS as usize;

        let n_bytes = if rem > 0 { div + 1 } else { div };

        (n_bytes, div, rem)
    }

    pub fn byte_size(n_bits: usize) -> usize {
        byte_size_extra(n_bits).0
    }

    #[cfg(test)]
    mod tests {
        use proptest::prelude::*;

        use super::*;

        fn decode_many_from_bytes(values: &[u8], n: usize) -> Result<Vec<bool>, DecodingError> {
            let mut bytes = Bytes::new(values);
            let mut output = Vec::new();
            decode_many(&mut bytes, n, &mut output)?;
            Ok(output)
        }

        fn recognize_many_from_bytes(
            values: &[u8],
            n: usize,
        ) -> Result<ByteSlice<'_, bool>, DecodingError> {
            let mut bytes = Bytes::new(values);
            recognize_many(&mut bytes, n)
        }

        proptest! {
            #[test]
            fn fuzz(bytes: Vec<u8>) {
                if !bytes.is_empty() {
                    let (n, bytes) = bytes.split_first().unwrap();
                    let n = *n as usize;
                    let decode_result = decode_many_from_bytes(bytes, n);
                    let recognize_result = recognize_many_from_bytes(bytes, n);
                    assert_eq!(decode_result.is_ok(), recognize_result.is_ok());
                }
            }
        }
    }
}

pub mod utils {
    use super::*;

    pub fn encode_discriminant<W: io::Write>(n: usize, writer: &mut W) -> io::Result<()> {
        let n = u8::try_from(n).expect("by default types can only have up to 255 items");
        writer.write_all(&[n])
    }

    pub fn decode_discriminant(bytes: &mut Bytes<'_>) -> Result<usize, DecodingError> {
        if bytes.len() > 0 {
            let n = bytes[0] as usize;
            bytes.consume(1);
            Ok(n)
        } else {
            Err(UnexpectedEof::new(bytes).into())
        }
    }

    pub fn encode_discriminant_relaxed<W: io::Write>(n: usize, writer: &mut W) -> io::Result<()> {
        super::uint::encode(n as u64, writer)
    }

    pub fn decode_discriminant_relaxed(bytes: &mut Bytes<'_>) -> Result<usize, DecodingError> {
        super::uint::decode(bytes).map(|n| n as usize)
    }

    pub fn byte_index_of(bit: usize) -> usize {
        bit / u8::BITS as usize
    }

    pub fn try_test_bit_at(i: usize, slice: &[u8]) -> Option<bool> {
        let (_, div, rem) = super::bool::byte_size_extra(i);

        if slice.len() > div {
            Some((slice[div] >> rem) & 0x01 != 0)
        } else {
            None
        }
    }

    pub fn test_bit_at(i: usize, slice: &[u8]) -> bool {
        try_test_bit_at(i, slice).expect("invalid slice for bit testing")
    }

    pub fn set_bit_at(i: usize, slice: &mut [u8]) {
        let (_, div, rem) = super::bool::byte_size_extra(i);

        if slice.len() > div {
            let mask = 1 << rem;
            slice[div] |= mask;
        } else {
            panic!("invalid slice for bit manipulation");
        }
    }

    pub fn clear_bit_at(i: usize, slice: &mut [u8]) {
        let (_, div, rem) = super::bool::byte_size_extra(i);

        if slice.len() > div {
            let mask = !(1 << rem);
            slice[div] &= mask;
        } else {
            panic!("invalid slice for bit manipulation");
        }
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use super::*;

        #[test]
        fn bit_testing() {
            assert!(test_bit_at(2, &[0b10000100]));
            assert!(!test_bit_at(2, &[0b10000000]));
            assert!(test_bit_at(8, &[0b10000100, 0b00000001]));
        }

        #[test]
        fn bit_testing_with_bool_encoding() {
            let mut cursor = Cursor::new(Vec::new());
            super::bool::encode_many(&[false, true], &mut cursor).unwrap();
            let bytes = cursor.into_inner();

            assert!(!test_bit_at(0, &bytes));
            assert!(test_bit_at(1, &bytes));
        }

        #[test]
        fn bit_manipulation() {
            let mut bytes = [0; 2];

            set_bit_at(2, &mut bytes);
            assert!(test_bit_at(2, &bytes));

            clear_bit_at(2, &mut bytes);
            assert!(!test_bit_at(2, &bytes));
        }

        #[test]
        fn byte_index() {
            assert_eq!(byte_index_of(0), 0);
            assert_eq!(byte_index_of(7), 0);
            assert_eq!(byte_index_of(8), 1);
        }
    }
}
