use serde_indexed::{DeserializeIndexed, SerializeIndexed};

/// buffer should be big enough to hold serialized object.
fn cbor_serialize<T: serde::Serialize>(
    object: &T,
    buffer: &mut [u8],
) -> Result<usize, serde_cbor::Error> {
    let writer = serde_cbor::ser::SliceWrite::new(buffer);
    let mut ser = serde_cbor::Serializer::new(writer);

    object.serialize(&mut ser)?;

    let writer = ser.into_inner();
    let size = writer.bytes_written();

    Ok(size)
}

/// may or may not modify buffer to hold temporary data.
/// buffer may be longer than serialized T.
fn cbor_deserialize<'de, T: serde::Deserialize<'de>>(
    buffer: &'de mut [u8],
) -> Result<T, serde_cbor::Error> {
    let mut deserializer = serde_cbor::de::Deserializer::from_mut_slice(buffer);
    serde::Deserialize::deserialize(&mut deserializer)
}

/// scratch should be big enough to hold temporary data.
/// buffer must not have trailing data.
fn cbor_deserialize_with_scratch<'de, T: serde::Deserialize<'de>>(
    buffer: &'de [u8],
    scratch: &'de mut [u8],
) -> Result<T, serde_cbor::Error> {
    serde_cbor::de::from_slice_with_scratch(buffer, scratch)
}

mod some_keys {
    use super::*;

    use hex_literal::hex;

    #[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
    #[serde_indexed(offset = 1)]
    pub struct SomeKeys {
        pub number: i32,
        pub bytes: [u8; 7],
        pub string: heapless::String<10>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub option: Option<u8>,
        pub vector: heapless::Vec<u8, 16>,
    }

    #[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
    // #[serde_indexed(offset = 1)]
    pub struct NakedOption {
        pub option: Option<SomeKeys>,
        pub num: usize,
        pub key: bool,
    }

    #[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
    // #[serde_indexed(offset = 1)]
    pub struct EmptyStruct {}

    fn an_example() -> (&'static [u8], SomeKeys) {
        let mut string = heapless::String::new();
        string.push_str("so serde").unwrap();

        let mut vector = heapless::Vec::<u8, 16>::new();
        vector.push(42).unwrap();

        let value = SomeKeys {
            number: -7,
            bytes: [37u8; 7],
            string,
            option: None,
            vector,
        };
        // in Python: cbor.dumps({1: -7, 2: [37]*7, 3: "so serde", 5: [42]*1})
        let serialized: &[u8] =
            &hex!("a40126028718251825182518251825182518250368736f2073657264650581182a");
        (serialized, value)
    }

    fn another_example() -> (&'static [u8], SomeKeys) {
        let (_, mut an_example) = an_example();
        an_example.option = Some(0xff);
        // in Python: cbor.dumps({1: -7, 2: [37]*7, 3: "so serde", 4: 0xff, 5: [42]*1})
        let serialized: &[u8] =
            &hex!("a50126028718251825182518251825182518250368736f2073657264650418ff0581182a");
        (serialized, an_example)
    }

    #[test]
    fn serialize() {
        let (serialized_value, example) = an_example();

        let mut buffer = [0u8; 64];
        let size = cbor_serialize(&example, &mut buffer).unwrap();

        assert_eq!(&buffer[..size], serialized_value);
    }

    #[test]
    fn deserialize() {
        let (serialized_value, example) = an_example();

        // no allocations need in this case.
        let maybe_example: SomeKeys =
            cbor_deserialize_with_scratch(serialized_value, &mut []).unwrap();

        assert_eq!(maybe_example, example);
    }

    #[test]
    fn another_serialize() {
        let (serialized_value, example) = another_example();

        let mut buffer = [0u8; 64];
        let size = cbor_serialize(&example, &mut buffer).unwrap();

        assert_eq!(&buffer[..size], serialized_value);
    }

    #[test]
    fn another_deserialize() {
        let (serialized_value, example) = another_example();
        // could also use `cbor_deserialize_with_scratch` in this case,
        // demonstrating the `cbor_deserialize` function.
        let mut buffer = serialized_value.to_owned();

        let maybe_example: SomeKeys = cbor_deserialize(&mut buffer).unwrap();

        assert_eq!(maybe_example, example);
    }
}
