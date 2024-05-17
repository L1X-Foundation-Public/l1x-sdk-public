use borsh::{BorshDeserialize, BorshSerialize};
use macropol;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uint::construct_uint;

#[macropol::macropol]
macro_rules! impl_str_type {
    ($iden: ident, $ty: tt) => {
        /// [`$&iden`] same as [`$&ty`] but JSON serializer serializes it to a string. The origninal [`$&ty`] value can be accessed by `$&iden.0`
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            BorshDeserialize,
            BorshSerialize,
            Default,
        )]
        pub struct $iden(pub $ty);

        impl From<$ty> for $iden {
            fn from(v: $ty) -> Self {
                Self(v)
            }
        }

        impl From<$iden> for $ty {
            fn from(v: $iden) -> $ty {
                v.0
            }
        }

        impl Serialize for $iden {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $iden {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                let s: String = Deserialize::deserialize(deserializer)?;
                Ok(Self(str::parse::<$ty>(&s).map_err(|err| {
                    serde::de::Error::custom(err.to_string())
                })?))
            }
        }
    };
}

impl_str_type!(U128, u128);
impl_str_type!(U64, u64);
impl_str_type!(I128, i128);
impl_str_type!(I64, i64);

construct_uint! {
    /// `U256` type implementation.
    ///
    /// The type is implemented with [`uint::construct_uint`] crate but serialized to JSON as a decimal string.
    #[derive(BorshDeserialize, BorshSerialize)]
    pub struct U256(4);
}

impl Serialize for U256 {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Self::from_dec_str(s.as_str())
            .map_err(|err| serde::de::Error::custom(err.to_string()))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_serde {
        ($str_type: tt, $int_type: tt, $number: expr) => {
            let a: $int_type = $number;
            let str_a: $str_type = a.into();
            let b: $int_type = str_a.into();
            assert_eq!(a, b);

            let str: String = serde_json::to_string(&str_a).unwrap();
            let deser_a: $str_type = serde_json::from_str(&str).unwrap();
            assert_eq!(a, deser_a.0);
        };
    }

    macro_rules! test_serde_u256 {
        ($number: expr) => {
            let num = $number;
            let str_a = serde_json::to_string(&U256::from(num)).unwrap();
            assert_eq!(str_a, "\"".to_string() + &num.to_string() + "\"");

            let deser_a: U256 = serde_json::from_str(&str_a).unwrap();
            assert_eq!(deser_a, num.into());
        };
    }

    #[test]
    fn test_u256() {
        test_serde_u256!(0);
        test_serde_u256!(1);
        test_serde_u256!(123);
        test_serde_u256!(10u128.pow(18));
        test_serde_u256!(2u128.pow(100));
        test_serde_u256!(u128::max_value());
        test_serde_u256!(U256::max_value());
    }

    #[test]
    fn test_u128() {
        test_serde!(U128, u128, 0);
        test_serde!(U128, u128, 1);
        test_serde!(U128, u128, 123);
        test_serde!(U128, u128, 10u128.pow(18));
        test_serde!(U128, u128, 2u128.pow(100));
        test_serde!(U128, u128, u128::max_value());
    }

    #[test]
    fn test_i128() {
        test_serde!(I128, i128, 0);
        test_serde!(I128, i128, 1);
        test_serde!(I128, i128, -1);
        test_serde!(I128, i128, 123);
        test_serde!(I128, i128, 10i128.pow(18));
        test_serde!(I128, i128, 2i128.pow(100));
        test_serde!(I128, i128, -(2i128.pow(100)));
        test_serde!(I128, i128, i128::max_value());
        test_serde!(I128, i128, i128::min_value());
    }

    #[test]
    fn test_u64() {
        test_serde!(U64, u64, 0);
        test_serde!(U64, u64, 1);
        test_serde!(U64, u64, 123);
        test_serde!(U64, u64, 10u64.pow(18));
        test_serde!(U64, u64, 2u64.pow(60));
        test_serde!(U64, u64, u64::max_value());
    }

    #[test]
    fn test_i64() {
        test_serde!(I64, i64, 0);
        test_serde!(I64, i64, 1);
        test_serde!(I64, i64, -1);
        test_serde!(I64, i64, 123);
        test_serde!(I64, i64, 10i64.pow(18));
        test_serde!(I64, i64, 2i64.pow(60));
        test_serde!(I64, i64, -(2i64.pow(60)));
        test_serde!(I64, i64, i64::max_value());
        test_serde!(I64, i64, i64::min_value());
    }
}
