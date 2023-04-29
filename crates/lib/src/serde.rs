pub mod set {
    use fixed_map::raw::RawStorage;
    use fixed_map::{Key, Set};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[inline]
    pub fn serialize<S, T>(set: &Set<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Serialize,
    {
        set.as_raw().serialize(serializer)
    }

    #[inline]
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Set<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Key,
        T::SetStorage: RawStorage,
        <T::SetStorage as RawStorage>::Value: Deserialize<'de>,
    {
        Ok(Set::from_raw(
            <T::SetStorage as RawStorage>::Value::deserialize(deserializer)?,
        ))
    }
}
