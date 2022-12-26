use derive_more::Display;
use fnv::FnvHasher;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
};

/// RNG seed to use for all randomized processes during world gen.
///
/// When deserialiing, this type supports a few options:
/// - If the value is an integer that fits into `u64`, use that value
/// - If it's a string that can be parsed into a `u64`, use the parsed value
/// - If it's any other string, just keep the string
/// - If it's anything else (out of range number, float, array, etc.), error
///
/// **Note:** It seems that some serde implementations (including
/// serde_json) will be overzealous and accidentally support additional
/// data types here. E.g. if you pass a bool, it will stringify it then
/// hash the string. Don't consider that supported behavior, just a
/// bug.
///
/// After deserialization, the type will be stored either as the interger value
/// or as the string value. When it comes time to use the seed, if it is a
/// string, it will be converted to a `u64` via hashing.
///
/// Regardless of how the seed value is input, it will always be serialized
/// as a **string**. JSON and TOML don't allow 64-bit unsigned integers,
/// so certain seeds can have issues if the value is serialized as a
/// number. By serializing as a string, we avoid that, and the seed
/// will still be parsed back into the same number next time it is
/// deserialized.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum Seed {
    /// An integer seed, which can be used directly
    Int(u64),
    /// A textual string, which will be hashed into a u64 before use
    Text(String),
}

impl Seed {
    /// Convert the seed to a `u64`, so it can actually be used in an RNG
    /// machine
    pub fn to_u64(&self) -> u64 {
        match self {
            Self::Int(seed) => *seed,
            Self::Text(text) => {
                let mut hasher = FnvHasher::default();
                text.hash(&mut hasher);
                hasher.finish()
            }
        }
    }
}

impl From<u64> for Seed {
    fn from(seed: u64) -> Self {
        Self::Int(seed)
    }
}

// Convert a string to a seed. If possible, parse it as an int. Otherwise,
// store the raw text, to be hashed later
impl From<&str> for Seed {
    fn from(seed_str: &str) -> Self {
        match seed_str.parse::<u64>() {
            Ok(seed) => Self::Int(seed),
            Err(_) => Self::Text(seed_str.into()),
        }
    }
}

impl From<&Seed> for u64 {
    fn from(seed: &Seed) -> Self {
        seed.to_u64()
    }
}

impl Serialize for Seed {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Always serialize a seed as a string, to avoid issues with large ints
        serializer.serialize_str(&self.to_string())
    }
}

// Custom deserialization to handle both int and string variants
impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        // We can deserialize from a bunch of different types so we can't give
        // a type hint here
        deserializer.deserialize_any(SeedVisitor)
    }
}

/// Macro to make it easier to implement visit logic for different types
macro_rules! impl_visit {
    ($fname:ident, $type:ty) => {
        fn $fname<E>(self, value: $type) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value
                .try_into()
                .map(Seed::Int)
                .map_err(|_| E::custom(format!("u64 out of range: {}", value)))
        }
    };
}

struct SeedVisitor;

impl<'de> Visitor<'de> for SeedVisitor {
    type Value = Seed;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a positive integer or string")
    }

    // yay for metaprogramming
    impl_visit!(visit_u8, u8);
    impl_visit!(visit_u16, u16);
    impl_visit!(visit_u32, u32);
    impl_visit!(visit_u64, u64);
    impl_visit!(visit_u128, u128);
    impl_visit!(visit_i8, i8);
    impl_visit!(visit_i16, i16);
    impl_visit!(visit_i32, i32);
    impl_visit!(visit_i64, i64);
    impl_visit!(visit_i128, i128);

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // This will try to parse as an int, then fall back to string variant
        Ok(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{
        assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, Token,
    };

    /// Test parsing seeds
    #[test]
    fn test_from_str() {
        // Valid u64 -> parses as an int
        assert_eq!(Seed::from("0"), Seed::Int(0));
        assert_eq!(
            Seed::from("12506774975058000"),
            Seed::Int(12506774975058000)
        );

        // Invalid u64 -> stores the raw text
        assert_eq!(Seed::from("-1"), Seed::Text("-1".into()));
        assert_eq!(Seed::from("potato"), Seed::Text("potato".into()));
    }

    /// Test converting seeds to an int
    #[test]
    fn test_to_u64() {
        // Int -> just use the value
        assert_eq!(Seed::Int(0).to_u64(), 0);
        assert_eq!(Seed::Int(12506774975058000).to_u64(), 12506774975058000);

        // Text -> string gets hashed
        assert_eq!(Seed::Text("-1".into()).to_u64(), 16020590405669718844);
        assert_eq!(Seed::Text("potato".into()).to_u64(), 6265489318014208823);
    }

    /// Test serialization of seeds
    #[test]
    fn test_serialize() {
        // Int -> gets stringified (to avoid overflow issues)
        assert_ser_tokens(&Seed::Int(0), &[Token::String("0")]);
        assert_ser_tokens(
            &Seed::Int(12506774975058000),
            &[Token::String("12506774975058000")],
        );

        // Text -> use the string
        assert_ser_tokens(&Seed::Text("-1".into()), &[Token::String("-1")]);
        assert_ser_tokens(
            &Seed::Text("potato".into()),
            &[Token::String("potato")],
        );
    }

    /// Test deserialization of seeds
    #[test]
    fn test_deserialize() {
        // Int -> parse the int
        assert_de_tokens(&Seed::Int(0), &[Token::String("0")]);
        assert_de_tokens(
            &Seed::Int(12506774975058000),
            &[Token::String("12506774975058000")],
        );

        // Text -> use the string (including numbers that fail to parse)
        assert_de_tokens(&Seed::Text("-1".into()), &[Token::String("-1")]);
        assert_ser_tokens(
            &Seed::Text("potato".into()),
            &[Token::String("potato")],
        );

        // Invalid input type -> error
        assert_de_tokens_error::<Seed>(
            &[Token::I32(-1)],
            "u64 out of range: -1",
        );
        assert_de_tokens_error::<Seed>(
            &[Token::F32(1.0)],
            "invalid type: floating point `1`, \
            expected a positive integer or string",
        );
        assert_de_tokens_error::<Seed>(
            &[Token::Bool(false)],
            "invalid type: boolean `false`, \
            expected a positive integer or string",
        );
    }
}
