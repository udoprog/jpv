pub mod set {
    #![allow(clippy::extra_unused_type_parameters)]

    use fixed_map::raw::RawStorage;
    use fixed_map::{Key, Set};
    use musli::de::{Decode, Decoder};
    use musli::en::{Encode, Encoder};
    use musli::mode::Mode;
    use musli::Context;

    #[inline]
    pub fn encode<M, C, E, T>(set: &Set<T>, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        M: Mode,
        E: Encoder,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Encode,
    {
        set.as_raw().encode(cx, encoder)
    }

    #[inline]
    pub fn decode<'de, M, C, D, T>(cx: &mut C, decoder: D) -> Result<Set<T>, C::Error>
    where
        C: Context<Input = D::Error>,
        M: Mode,
        D: Decoder<'de>,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Decode<'de, M>,
    {
        Ok(Set::from_raw(<T::SetStorage as RawStorage>::Value::decode(
            cx, decoder,
        )?))
    }
}
