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
    use serde_byte_array::ByteArray;
    use serde_bytes::Bytes;

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
    #[serde_indexed(offset = 1)]
    pub struct SomeRefKeys<'a, 'b, 'c> {
        pub number: i32,
        pub bytes: &'a ByteArray<7>,
        pub string: &'b str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub option: Option<u8>,
        pub vector: &'c Bytes,
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
    pub struct NakedRefOption<'a, 'b, 'c> {
        pub option: Option<SomeRefKeys<'a, 'b, 'c>>,
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
        // in Python: cbor2.dumps({1: -7, 2: [37]*7, 3: "so serde", 5: [42]*1})
        let serialized: &[u8] =
            &hex!("a40126028718251825182518251825182518250368736f2073657264650581182a");
        (serialized, value)
    }

    fn a_ref_example() -> (&'static [u8], SomeRefKeys<'static, 'static, 'static>) {
        const BYTE_ARRAY: ByteArray<7> = ByteArray::new([37u8; 7]);
        let value = SomeRefKeys {
            number: -7,
            bytes: &BYTE_ARRAY,
            string: "so serde",
            option: None,
            vector: Bytes::new(&[42]),
        };
        // in Python: cbor2.dumps({1: -7, 2: bytes([37]*7), 3: "so serde", 5: bytes([42]*1)}).
        let serialized: &[u8] = &hex!("a401260247252525252525250368736f20736572646505412a");
        (serialized, value)
    }

    fn another_example() -> (&'static [u8], SomeKeys) {
        let (_, mut an_example) = an_example();
        an_example.option = Some(0xff);
        // in Python: cbor2.dumps({1: -7, 2: [37]*7, 3: "so serde", 4: 0xff, 5: [42]*1})
        let serialized: &[u8] =
            &hex!("a50126028718251825182518251825182518250368736f2073657264650418ff0581182a");
        (serialized, an_example)
    }

    fn another_ref_example() -> (&'static [u8], SomeRefKeys<'static, 'static, 'static>) {
        let (_, mut an_example) = a_ref_example();
        an_example.option = Some(0xff);
        // in Python: cbor2.dumps({1: -7, 2: bytes([37]*7), 3: "so serde", 4: 0xff,  5: bytes([42]*1)}).hex()
        let serialized: &[u8] = &hex!("a501260247252525252525250368736f2073657264650418ff05412a");
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
    fn serialize_ref() {
        let (serialized_value, example) = a_ref_example();

        let mut buffer = [0u8; 64];
        let size = cbor_serialize(&example, &mut buffer).unwrap();

        assert_eq!(&buffer[..size], serialized_value);
    }

    #[test]
    fn deserialize_ref() {
        let (serialized_value, example) = a_ref_example();

        // no allocations need in this case.
        let maybe_example: SomeRefKeys =
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

    #[test]
    fn another_ref_serialize() {
        let (serialized_value, example) = another_ref_example();

        let mut buffer = [0u8; 64];
        let size = cbor_serialize(&example, &mut buffer).unwrap();

        assert_eq!(&buffer[..size], serialized_value);
    }

    #[test]
    fn another_ref_deserialize() {
        let (serialized_value, example) = another_ref_example();
        // could also use `cbor_deserialize_with_scratch` in this case,
        // demonstrating the `cbor_deserialize` function.
        let mut buffer = serialized_value.to_owned();

        let maybe_example: SomeRefKeys = cbor_deserialize(&mut buffer).unwrap();

        assert_eq!(maybe_example, example);
    }
}

mod cow {
    use super::*;
    use std::borrow::Cow;

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

    const SERIALIZED_LIFETIME_EXAMPLE: &[u8] = b"\xa1\x01\x83\x01\x02\x03";

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
}

mod generics {
    use super::*;
    use heapless::String;
    use serde_byte_array::ByteArray;
    use serde_bytes::Bytes;

    const SERIALIZED_GENERIC_EXAMPLE: &'static [u8] = b"\xa1\x01\x43\x01\x02\x03";

    #[derive(PartialEq, Debug, SerializeIndexed, DeserializeIndexed)]
    #[serde_indexed(offset = 1)]
    struct WithGeneric<T> {
        data: T,
        #[serde(skip_serializing_if = "Option::is_none")]
        option: Option<u8>,
    }

    fn generics_example<'a>() -> WithGeneric<&'a Bytes> {
        WithGeneric {
            data: Bytes::new(&[1, 2, 3]),
            option: None,
        }
    }

    #[derive(PartialEq, Debug, SerializeIndexed, DeserializeIndexed)]
    #[serde_indexed(offset = 1)]
    struct WithConstGeneric<const N: usize> {
        data: ByteArray<N>,
        #[serde(skip_serializing_if = "Option::is_none")]
        option: Option<u8>,
    }

    fn const_generics_example<'a>() -> WithConstGeneric<3> {
        WithConstGeneric {
            data: ByteArray::new([1, 2, 3]),
            option: None,
        }
    }

    #[test]
    fn serialize() {
        let data = generics_example();
        let mut buf = [0u8; 64];
        let size = cbor_serialize(&data, &mut buf).unwrap();

        assert_eq!(&buf[..size], SERIALIZED_GENERIC_EXAMPLE);

        let data = const_generics_example();
        let mut buf = [0u8; 64];
        let size = cbor_serialize(&data, &mut buf).unwrap();

        assert_eq!(&buf[..size], SERIALIZED_GENERIC_EXAMPLE);
    }

    #[test]
    fn deserialize() {
        let example = generics_example();

        let deserialized: WithGeneric<&'_ Bytes> =
            cbor_deserialize_with_scratch(SERIALIZED_GENERIC_EXAMPLE, &mut []).unwrap();

        assert_eq!(deserialized, example);

        let example = const_generics_example();

        let deserialized: WithConstGeneric<3> =
            cbor_deserialize_with_scratch(SERIALIZED_GENERIC_EXAMPLE, &mut []).unwrap();

        assert_eq!(deserialized, example);
    }

    #[derive(PartialEq, Debug, SerializeIndexed, DeserializeIndexed)]
    #[serde_indexed(offset = 1)]
    struct WithAllGenerics<'a, 'b, T, I, const N: usize, const Z: usize> {
        data1: heapless::Vec<T, N>,
        data2: heapless::Vec<I, Z>,
        data3: &'a Bytes,
        data4: &'b ByteArray<Z>,
    }

    fn all_generics_example<'a, 'b>() -> WithAllGenerics<'a, 'b, String<5>, u8, 10, 3> {
        let data1 = heapless::Vec::from_slice(&["abc".into(), "acdef".into()]).unwrap();
        let data2 = heapless::Vec::from_slice(&[1, 2]).unwrap();

        const BYTES: ByteArray<3> = ByteArray::new(*b"123");
        WithAllGenerics {
            data1,
            data2,
            data3: Bytes::new(b"bytes"),
            data4: &BYTES,
        }
    }

    #[test]
    fn all_generics() {
        const SERIALIZED_ALL_GENERIC_EXAMPLE: &'static [u8] = b"\xa4\x01\x82\x63\x61\x62\x63\x65\x61\x63\x64\x65\x66\x02\x82\x01\x02\x03\x45\x62\x79\x74\x65\x73\x04\x43\x31\x32\x33";
        let data = all_generics_example();
        let mut buf = [0u8; 64];
        let size = cbor_serialize(&data, &mut buf).unwrap();

        println!("{buf:02x?}");
        assert_eq!(&buf[..size], SERIALIZED_ALL_GENERIC_EXAMPLE);

        let example = all_generics_example();

        let deserialized: WithAllGenerics<'_, '_, String<5>, u8, 10, 3> =
            cbor_deserialize_with_scratch(SERIALIZED_ALL_GENERIC_EXAMPLE, &mut []).unwrap();

        assert_eq!(deserialized, example);
    }
}
