use std::{marker::PhantomData, ops::Deref};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bytes<'a> {
    bytes: &'a [u8],
    consumed: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self::with_consumed(bytes, 0)
    }

    pub(crate) fn with_consumed(bytes: &'a [u8], consumed: usize) -> Self {
        Self { bytes, consumed }
    }

    pub fn consume(&mut self, len: usize) {
        self.consumed += len;
    }

    pub fn consume_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        if self.consumed + len <= self.bytes.len() {
            let offset = self.consumed;
            self.consumed += len;
            Some(&self.bytes[offset..self.consumed])
        } else {
            None
        }
    }

    pub fn consume_slice<T>(&mut self, len: usize) -> Option<ByteSlice<'a, T>> {
        let offset = self.consumed;

        self.consume_bytes(len).map(|slice| ByteSlice {
            slice,
            offset,
            ty: PhantomData,
        })
    }

    pub fn take_slice_from<T>(&self, offset: Mark) -> ByteSlice<'a, T> {
        let offset = offset.to_usize();
        let slice = &self.bytes[offset..self.consumed];

        ByteSlice {
            slice,
            offset,
            ty: PhantomData,
        }
    }

    pub fn mark(&self) -> Mark {
        Mark(self.consumed)
    }
}

impl Deref for Bytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes[self.consumed..]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mark(usize);

impl Mark {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<Mark> for usize {
    fn from(mark: Mark) -> Self {
        mark.to_usize()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteSlice<'a, T> {
    slice: &'a [u8],
    offset: usize,
    ty: PhantomData<fn() -> T>,
}

impl<'a, T> ByteSlice<'a, T> {
    pub fn as_bytes(&self) -> &[u8] {
        self.slice
    }

    pub fn span(&self) -> ByteSpan<T> {
        ByteSpan {
            offset: self.offset,
            len: self.slice.len(),
            ty: self.ty,
        }
    }

    pub fn cast<U>(self) -> ByteSlice<'a, U> {
        ByteSlice {
            slice: self.slice,
            offset: self.offset,
            ty: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteSpan<T> {
    offset: usize,
    len: usize,
    ty: PhantomData<fn() -> T>,
}

impl<T> ByteSpan<T> {
    pub fn as_bytes<'a>(&self, origin: &'a [u8]) -> &'a [u8] {
        let end = self.offset + self.len;
        &origin[self.offset..end]
    }

    pub fn cast<U>(self) -> ByteSpan<U> {
        ByteSpan {
            offset: self.offset,
            len: self.len,
            ty: PhantomData,
        }
    }
}
