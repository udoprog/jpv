pub mod set {
    use fixed_map::key::Key;
    use fixed_map::Set;
    use musli::de::{Decode, Decoder, SequenceDecoder};
    use musli::en::{Encode, Encoder, SequenceEncoder};
    use musli::mode::Mode;

    pub fn encode<M, E, T>(set: &Set<T>, encoder: E) -> Result<E::Ok, E::Error>
    where
        M: Mode,
        E: Encoder,
        T: Key + Encode,
    {
        let mut seq = encoder.encode_sequence(set.len())?;

        for value in set.iter() {
            value.encode(seq.next()?)?;
        }

        seq.end()
    }

    pub fn decode<'de, M, D, T>(decoder: D) -> Result<Set<T>, D::Error>
    where
        M: Mode,
        D: Decoder<'de>,
        T: Key + Decode<'de, M>,
    {
        let mut access = decoder.decode_sequence()?;
        let mut out = Set::new();

        while let Some(value) = access.next()? {
            out.insert(T::decode(value)?);
        }

        Ok(out)
    }
}
