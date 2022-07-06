use std::io::Cursor;

use phenix_runtime::{
    bytes::{ByteSpan, Bytes},
    Decodable, DecodingError, Encodable,
};

#[derive(Encodable, Decodable)]
#[phenix_runtime::by_parts]
pub struct Message {
    pub from: String,
    pub to: String,
    pub text: String,
}

#[derive(Debug)]
struct LazyMessage {
    pub from: String,
    pub to: String,
    // It is possible to store the ByteSlice<'a, T> directly, but that requires
    // a lifetime.
    text: ByteSpan<String>,
}

impl LazyMessage {
    pub fn new(bytes: &[u8]) -> Result<Self, DecodingError> {
        let mut from = None;
        let mut to = None;
        let mut text = None;

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();

        for part in Message::recognize_by_parts(&mut Bytes::new(bytes), &mut buf1) {
            match part? {
                // Decode short presumably strings eagerly.
                MessagePart::From(part) => from = Some(part.decode(&mut buf2)?),
                MessagePart::To(part) => to = Some(part.decode(&mut buf2)?),
                // Store the byte location of presumably long string for lazy
                // decoding.
                MessagePart::Text(part) => text = Some(part.span()),
            }
        }

        Ok(LazyMessage {
            from: from.unwrap(),
            to: to.unwrap(),
            text: text.unwrap(),
        })
    }

    pub fn text(&self, bytes: &[u8]) -> String {
        self.text.decode(bytes, &mut Vec::new()).unwrap()
    }
}

fn main() {
    let message = Message {
        from: "Felix".to_string(),
        to: "Aurelia".to_string(),
        text: "Very long message".to_string(),
    };

    let mut cursor = Cursor::new(Vec::new());
    message.encode(&mut cursor).unwrap();

    let bytes = cursor.into_inner();
    let lazy = LazyMessage::new(&bytes).unwrap();

    println!("from: {}", lazy.from);
    println!("to: {}", lazy.to);
    println!("text: {}", lazy.text(&bytes));
}
