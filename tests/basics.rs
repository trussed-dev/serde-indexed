use serde_indexed::{DeserializeIndexed, SerializeIndexed};

mod some_keys {
    use super::*;

    use heapless::consts;

    #[derive(Clone, Debug, PartialEq, SerializeIndexed, DeserializeIndexed)]
    // #[serde_indexed(offset = 1)]
    pub struct SomeKeys {
        pub number: i32,
        pub bytes: [u8; 7],
        pub string: heapless::String<consts::U10>,
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
            vector,
        }
    }

    // in Python: cbor.dumps({1: -7, 2: [37]*7, 3: "so serde", 4: [42]*1})
    const SERIALIZED_AN_EXAMPLE: &'static [u8] =
        b"\xa4\x01&\x02\x87\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x03hso serde\x04\x81\x18*";

    #[test]
    fn serialize() {
        let example = an_example();

        let mut buffer = [0u8; 1024];
        let writer = serde_cbor::ser::SliceWrite::new(&mut buffer);
        let mut ser = serde_cbor::Serializer::new(writer);

        use serde::ser::Serialize;

        example.serialize(&mut ser).unwrap();

        let writer = ser.into_inner();
        let size = writer.bytes_written();

        assert_eq!(&buffer[..size], SERIALIZED_AN_EXAMPLE);
    }

    #[test]
    fn deserialize() {
        let mut buffer = [0u8; 1024];
        buffer[..SERIALIZED_AN_EXAMPLE.len()].copy_from_slice(SERIALIZED_AN_EXAMPLE);

        let mut deserializer = serde_cbor::de::Deserializer::from_mut_slice(&mut buffer);
        let maybe_example: SomeKeys =
            serde::de::Deserialize::deserialize(&mut deserializer).unwrap();

        assert_eq!(maybe_example, an_example());
    }
}
