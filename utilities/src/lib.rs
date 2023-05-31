/// buffer should be big enough to hold serialized object.
pub fn cbor_serialize<T: serde::Serialize>(
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
pub fn cbor_deserialize<'de, T: serde::Deserialize<'de>>(
    buffer: &'de mut [u8],
) -> Result<T, serde_cbor::Error> {
    let mut deserializer = serde_cbor::de::Deserializer::from_mut_slice(buffer);
    serde::Deserialize::deserialize(&mut deserializer)
}

/// scratch should be big enough to hold temporary data.
/// buffer must not have trailing data.
pub fn cbor_deserialize_with_scratch<'de, T: serde::Deserialize<'de>>(
    buffer: &'de [u8],
    scratch: &'de mut [u8],
) -> Result<T, serde_cbor::Error> {
    serde_cbor::de::from_slice_with_scratch(buffer, scratch)
}

