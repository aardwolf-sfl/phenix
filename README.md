# phenix

**Phenix** is schema-based, language-neutral<sup>1</sup> encoding/decoding tool
with focus on compact format, lazy deserialization and support for stream-like
serialization. It targets design and implementation of stable-ish file formats
rather than extensible data interchange protocols, which has its consequences
(see goals and non-goals).

Supported languages:

* **Rust**
* C (encoding only)

<sup>1</sup> *The near-future focus will be on encoders, not necessarily decoders.*

## Example

```
struct Person {
    name: string,
    // Space-efficient encoding for integers and floats
    age: uint,
    // Encoded as bit vector
    working_hours: vector<bool>,
    // Encode new items one by one without knowing the final count
    projects: stream<string>,
}

enum Salutation {
    None,
    Some {
        // Enum variants can have data
        text: string,
    }
}

// Built-in bitflags type
flags Color {
    RED,
    GREEN,
    BLUE,
}
```

## Goals

* Convenient schema language from which the compiler can generate code for various programming languages
* Space-efficient encoding without reaching for generic, computationally expensive compression algorithms
* Lazy deserialization -- fast recognition of value's byte range without actually materializing it
* Stream-like serialization (and deserialization) -- continuously encode values one by one without the need of initializing the whole collection

## Non-goals

* Being the fastest in the world
* Rich and beautiful deserialization error reporting
* Backwards-compatible schema extensibility
* Support for generic serialization/deserialization frameworks like [serde](https://serde.rs/)
* Remote Procedure Call (RPC) system

## Alternatives

If you find yourself in need of the things mentioned in *non-goals*, consider
the following alternatives:

* [Protocol Buffers](https://developers.google.com/protocol-buffers)
* [Cap'n Proto](https://capnproto.org)
* [Postcard](https://github.com/jamesmunns/postcard)
* [Bincode](https://github.com/bincode-org/bincode)

## Status

**The project is in very early stage of development.** I have implemented
encoding/decoding for a few standard types and its derivatives. There is a very
basic compiler and code generation to Rust. The compiler assumes valid input and
does not check errors (obviously, this will change in the future). There is no
documentation, I am still in the phase of proofing the concept and looking for
rough edges.

## License

Dual-licensed under [MIT](LICENSE) and [UNLICENSE](UNLICENSE). Feel free to use
it, contribute or spread the word.
