use serde_indexed::{DeserializeIndexed, SerializeIndexed};

mod some_keys {
    use super::*;

    use heapless::consts;

    #[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
    #[serde_indexed(offset = 1)]
    // #[serde_indexed(skip_nones)]
    pub struct SomeKeys {
        pub number: i32,
        pub bytes: [u8; 7],
        pub string: heapless::String<consts::U10>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub option: Option<u8>,
        pub vector: heapless::Vec<u8, consts::U16>,
    }

    fn an_example() -> SomeKeys {
        let mut string = heapless::String::new();
        string.push_str("so serde").unwrap();

        let mut vector = heapless::Vec::<u8, consts::U16>::new();
        vector.push(42).unwrap();

        SomeKeys {
            number: -7,
            bytes: [37u8; 7],
            string,
            option: None,
            vector,
        }
    }

    fn another_example() -> SomeKeys {
        let mut string = heapless::String::new();
        string.push_str("so serde").unwrap();

        let mut vector = heapless::Vec::<u8, consts::U16>::new();
        vector.push(42).unwrap();

        SomeKeys {
            number: -7,
            bytes: [37u8; 7],
            string,
            option: Some(0xff),
            vector,
        }
    }

    // in Python: cbor.dumps({1: -7, 2: [37]*7, 3: "so serde", 5: [42]*1})
    const SERIALIZED_AN_EXAMPLE: &'static [u8] =
        b"\xa4\x01&\x02\x87\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x03hso serde\x05\x81\x18*";

    const SERIALIZED_ANOTHER_EXAMPLE: &'static [u8] =
        b"\xa5\x01&\x02\x87\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x03hso serde\x04\x18\xff\x05\x81\x18*";

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

    fn cbor_deserialize<'de, T: serde::Deserialize<'de>>(
        buffer: &'de mut [u8],
    ) -> Result<T, serde_cbor::Error> {
        let mut deserializer = serde_cbor::de::Deserializer::from_mut_slice(buffer);
        serde::de::Deserialize::deserialize(&mut deserializer)
    }

    #[test]
    fn serialize() {
        let example = an_example();

        let mut buffer = [0u8; 1024];
        let size = cbor_serialize(&example, &mut buffer).unwrap();

        assert_eq!(&buffer[..size], SERIALIZED_AN_EXAMPLE);
    }

    #[test]
    fn deserialize() {
        let mut buffer = [0u8; 1024];
        buffer[..SERIALIZED_AN_EXAMPLE.len()].copy_from_slice(SERIALIZED_AN_EXAMPLE);

        let maybe_example: SomeKeys = cbor_deserialize(&mut buffer).unwrap();

        assert_eq!(maybe_example, an_example());
    }

    #[test]
    fn another_serialize() {
        let example = another_example();

        let mut buffer = [0u8; 1024];
        let size = cbor_serialize(&example, &mut buffer).unwrap();

        assert_eq!(&buffer[..size], SERIALIZED_ANOTHER_EXAMPLE);
    }

    #[test]
    fn another_deserialize() {
        let mut buffer = [0u8; 1024];
        buffer[..SERIALIZED_ANOTHER_EXAMPLE.len()].copy_from_slice(SERIALIZED_ANOTHER_EXAMPLE);

        let maybe_example: SomeKeys = cbor_deserialize(&mut buffer).unwrap();

        assert_eq!(maybe_example, another_example());
    }
}
