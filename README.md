## Overview

Derive Serialize and Deserialize that replaces struct keys with numerical indices.

**WIP**. Primary use case is to handle CTAP CBOR messages.

Missing features:
- `skip_serializing_if` for optional keys.
- configurable index offset

This is my attempt to learn proc-macros, I'm roughly following [`serde-repr`][serde-repr].

Tip: To "see" the generated code, run `cargo expand --test basics`.

[serde-repr]: https://github.com/dtolnay/serde-repr

#### License

<sup>`serde-indexed` is licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.</sup>
<br>
<sub>Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.</sub>
