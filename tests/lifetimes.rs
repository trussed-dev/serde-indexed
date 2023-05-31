use serde_indexed::{DeserializeIndexed, SerializeIndexed};
use utilities::{cbor_deserialize_with_scratch, cbor_serialize};

extern crate alloc;
use alloc::borrow::Cow;

#[derive(PartialEq, Debug, SerializeIndexed, DeserializeIndexed)]
#[serde_indexed(offset = 1)]
struct WithLifetimes<'a> {
    data: Cow<'a, [u8]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    option: Option<u8>,
}

fn lifetime_example<'a>() -> WithLifetimes<'a> {
    WithLifetimes {
        data: Cow::Borrowed(&[1, 2, 3]),
        option: None,
    }
}

const SERIALIZED_LIFETIME_EXAMPLE: &'static [u8] = b"\xa1\x01\x83\x01\x02\x03";

#[test]
fn serialize() {
    let data = lifetime_example();
    let mut buf = [0u8; 64];

    let size = cbor_serialize(&data, &mut buf).unwrap();

    assert_eq!(&buf[..size], SERIALIZED_LIFETIME_EXAMPLE);
}

#[test]
fn deserialize() {
    let example = lifetime_example();

    let deserialized: WithLifetimes<'_> =
        cbor_deserialize_with_scratch(SERIALIZED_LIFETIME_EXAMPLE, &mut []).unwrap();

    assert_eq!(deserialized, example);
    let Cow::Owned(_) = deserialized.data else {
        panic!("Expected deserialized data Cow::Owned");
    };
}
