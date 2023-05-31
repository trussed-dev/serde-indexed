use serde_indexed::SerializeIndexed;
use utilities::cbor_serialize;

#[derive(SerializeIndexed)]
#[serde_indexed(offset = 1)]
struct WithLifetimes<'a> {
    shared_data: &'a [u8],
        #[serde(skip_serializing_if = "Option::is_none")]
        option: Option<u8>,
}

fn lifetime_example<'a>() -> WithLifetimes<'a> {
    WithLifetimes {
        shared_data: &[1, 2, 3],
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
