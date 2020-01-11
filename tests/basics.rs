use serde_indexed::{/*DeserializeIndexed,*/ SerializeIndexed};

mod some_keys {
    use super::*;

    // use heapless::{
    //     consts,
    //     String,
    //     Vec,
    // };

    // TODO Does this import belong here or in the macro?

    // use serde::ser::SerializeMap;
    #[derive(Clone, Debug, PartialEq, SerializeIndexed)]
    // #[serde_indexed(offset = 1)]
    pub struct SomeKeys {
        pub number: i32,
        pub bytes: [u8; 12],
        // pub string: String<consts::U10>,
        // pub vector: Vec<u8, consts::U16>,
    }

    fn an_example() -> SomeKeys {
        SomeKeys { number: -7, bytes: [37u8; 12] }
    }

    // TODO: add some kind of bytes, e.g.
    // cbor.dumps({1: -7, 2: bytes([42]*12)}) = b'\xa2\x01&\x02L************'
    const SERIALIZED_AN_EXAMPLE: &'static [u8] =
        b"\xa2\x01&\x02\x8c\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x18%\x18%";

    #[test]
    fn test_serialize() {
        let sk = an_example();

        let mut buffer = [0u8; 1024];
        let writer = serde_cbor::ser::SliceWrite::new(&mut buffer);
        let mut ser = serde_cbor::Serializer::new(writer);

        use serde::ser::Serialize;

        // CURRENTLY: stack overflow, as it recursively calls itself
        sk.serialize(&mut ser).unwrap();

        let writer = ser.into_inner();
        let size = writer.bytes_written();

        assert_eq!(&buffer[..size], SERIALIZED_AN_EXAMPLE);
    }

    // #[test]
    // fn test_deserialize() {
    //     let sk = an_example();
    // }

}
