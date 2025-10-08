pub mod set {
    #![allow(clippy::extra_unused_type_parameters)]

    use fixed_map::raw::RawStorage;
    use fixed_map::{Key, Set};
    use musli::de::{Decode, Decoder};
    use musli::en::{Encode, Encoder};

    #[inline]
    pub fn encode<E, T>(set: &Set<T>, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Encode<E::Mode>,
    {
        encoder.encode(set.as_raw())
    }

    #[inline]
    pub fn decode<'de, D, T>(decoder: D) -> Result<Set<T>, D::Error>
    where
        D: Decoder<'de>,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Decode<'de, D::Mode, D::Allocator>,
    {
        Ok(Set::from_raw(
            decoder.decode::<<T::SetStorage as RawStorage>::Value>()?,
        ))
    }
}
