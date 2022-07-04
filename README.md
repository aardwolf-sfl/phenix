# phenix

**Phenix** is<sup>1</sup> schema-based, language-neutral<sup>2</sup>
encoding/decoding tool with focus on compact format, lazy deserialization and
support for stream-like serialization. It targets design and implementation of
stable-ish file formats rather than extensible data interchange protocols, which
has its consequences (see goals and non-goals).

Supported languages:

* **Rust**

<sup>1</sup> *At this point not really, but hopefully will.*

<sup>2</sup> *The near-future focus will be on encoders, not necessarily decoders.*

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
