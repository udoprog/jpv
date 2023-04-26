pub mod set {
    use fixed_map::raw::RawStorage;
    use fixed_map::{Key, Set};
    use musli::de::{Decode, Decoder};
    use musli::en::{Encode, Encoder};
    use musli::mode::Mode;

    #[inline]
    pub fn encode<M, E, T>(set: &Set<T>, encoder: E) -> Result<E::Ok, E::Error>
    where
        M: Mode,
        E: Encoder,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Encode,
    {
        set.as_raw().encode(encoder)
    }

    #[inline]
    pub fn decode<'de, M, D, T>(decoder: D) -> Result<Set<T>, D::Error>
    where
        M: Mode,
        D: Decoder<'de>,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Decode<'de, M>,
    {
        Ok(Set::from_raw(<T::SetStorage as RawStorage>::Value::decode(
            decoder,
        )?))
    }
}
