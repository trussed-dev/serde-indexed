## serde-indexed

[![crates.io][crates-image]][crates-link]
[![Documentation][docs-image]][docs-link]

Derivation of [`Serialize`][serialize] and [`Deserialize`][deserialize] that replaces struct keys with numerical indices.

Primary use case is to handle [CTAP CBOR][ctap-cbor] messages, in particular support for:
- [`skip_serializing_if`][skip-serializing-if] for optional keys
- configurable index `offset`

#### Example

```rust
#[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
#[serde_indexed(offset = 1)]
pub struct SomeKeys {
    pub number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<u8>,
    pub bytes: [u8; 7],
}
```

This was a nice opportunity to learn proc-macros, I roughly followed [`serde-repr`][serde-repr].

To see some generated code, run `cargo expand --test basics`.

[crates-image]: https://img.shields.io/crates/v/serde-indexed.svg?style=flat-square
[crates-link]: https://crates.io/crates/serde-indexed
[docs-image]: https://img.shields.io/badge/docs.rs-api-green?style=flat-square
[docs-link]: https://docs.rs/serde-indexed
[serialize]: https://docs.serde.rs/serde/ser/trait.Serialize.html
[deserialize]: https://docs.serde.rs/serde/de/trait.Deserialize.html
[ctap-cbor]: https://fidoalliance.org/specs/fido-v2.0-ps-20190130/fido-client-to-authenticator-protocol-v2.0-ps-20190130.html#ctap2-canonical-cbor-encoding-form
[skip-serializing-if]: https://serde.rs/field-attrs.html#skip_serializing_if
[serde-repr]: https://github.com/dtolnay/serde-repr

#### License

<sup>`serde-indexed` is licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.</sup>
<br>
<sub>Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.</sub>
