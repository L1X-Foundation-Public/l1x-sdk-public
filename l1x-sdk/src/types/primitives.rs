use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use hex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

type AddressArray = [u8; 20];

/// Balance is a type for storing amounts of L1X tokens, specified in Shekels.
pub type Balance = u128;

/// Gas
pub type Gas = u64;

/// Number of the block
pub type BlockNumber = u128;

/// Hash of the block
pub type BlockHash = [u8; 32];

/// Unix timestamp
pub type TimeStamp = u128;

/// The address in L1X blockchain. This is a 20 bytes array. Can be represented as a hex string.
///
/// Because these addresses should be validated, they have to be converted with [`TryFrom`]
/// # Examples:
/// ```
/// use l1x_sdk::types::Address;
///
/// let invalid_str = Address::try_from("invalid");
/// assert!(invalid_str.is_err());
///
/// let invalid_vec = Address::try_from(vec![1, 2, 3]);
/// assert!(invalid_vec.is_err());
///
/// let valid_str = Address::try_from("a11ce00000000000000000000000000000000000");
/// assert!(valid_str.is_ok());
///
/// let valid_vec = Address::try_from(vec![
///     0xa1, 0x1c, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
/// ]);
/// assert!(valid_vec.is_ok());
/// ```
#[derive(
    BorshSerialize, BorshDeserialize, Hash, BorshSchema, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Address(AddressArray);

impl Address {
    /// Returns the hex string representation of [`Address`]
    ///
    /// # Examples
    /// ```
    /// use l1x_sdk::types::Address;
    /// let address = Address::try_from("a11ce00000000000000000000000000000000000").unwrap();
    ///
    /// assert_eq!(address.to_vec(),
    ///     vec![0xa1, 0x1c, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// ```
    pub fn to_string(&self) -> String {
        hex::encode(&self.0)
    }

    /// Returns [`Vec`] of raw [`Address`] bytes
    ///
    /// # Examples
    /// ```
    /// use l1x_sdk::types::Address;
    /// let address = Address::try_from(vec![
    ///     0xa1, 0x1c, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    /// ]).unwrap();
    ///
    /// assert_eq!(address.to_string(), "a11ce00000000000000000000000000000000000");
    /// ```
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Returns a reference to the inner `[u8; 20]` array
    pub fn as_bytes(&self) -> &AddressArray {
        &self.0
    }

    #[cfg(test)]
    pub fn test_create_address(address: &Vec<u8>) -> Self {
        let address: AddressArray = address.clone().try_into().unwrap();
        Address(address)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&hex::encode(self.0), f)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&hex::encode(self.0), f)
    }
}

impl From<AddressArray> for Address {
    fn from(address: AddressArray) -> Self {
        Self(address)
    }
}

impl From<&AddressArray> for Address {
    fn from(address: &AddressArray) -> Self {
        Self(address.clone())
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();
        match <AddressArray>::try_from(value) {
            Ok(address) => Ok(Self(address)),
            Err(_) => Err(format!(
                "Can't create address from vector or slice length={}",
                len
            )),
        }
    }
}

impl TryFrom<Vec<u8>> for Address {
    type Error = String;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<&Vec<u8>> for Address {
    type Error = String;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl TryFrom<&str> for Address {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.strip_prefix("0x").unwrap_or(value);
        match hex::decode(value) {
            Ok(val) => match <AddressArray>::try_from(val) {
                Ok(address) => Ok(Self(address)),
                Err(_) => Err(format!("Can't create address from vector")),
            },
            Err(_) => Err(format!("Can't create address from string {}", value)),
        }
    }
}

impl TryFrom<String> for Address {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&String> for Address {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Address::try_from(s).map_err(|err| serde::de::Error::custom(err))?)
    }
}

#[cfg(test)]
mod test {
    use crate::types::Address;
    use std::fmt;

    use super::AddressArray;

    #[test]
    pub fn address_try_from() {
        let addr_vec: Vec<u8> = vec![
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x00, 0x11, 0x22, 0x33, 0x44,
        ];
        let addr_str = hex::encode(addr_vec.clone());
        let addr_0x_str = hex::encode(addr_vec.clone());
        let test_addr = Address::test_create_address(&addr_vec);

        assert_eq!(Address::try_from(addr_str.clone()), Ok(test_addr));
        assert_eq!(Address::try_from(addr_str.as_str()), Ok(test_addr));
        assert_eq!(Address::try_from(addr_0x_str.clone()), Ok(test_addr));
        assert_eq!(Address::try_from(addr_0x_str.as_str()), Ok(test_addr));
        assert_eq!(Address::try_from(&addr_vec), Ok(test_addr));
        assert_eq!(Address::try_from(addr_vec.clone()), Ok(test_addr));
        assert_eq!(Address::try_from(addr_vec.as_slice()), Ok(test_addr));
    }

    #[test]
    pub fn address_try_from_incorrect() {
        let addr_vec_too_short: Vec<u8> = vec![
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x00, 0x11, 0x22, 0x33,
        ];
        let addr_vec_too_long: Vec<u8> = vec![
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0xff,
        ];
        let addr_str_short = hex::encode(addr_vec_too_short.clone());
        let addr_str_long = hex::encode(addr_vec_too_long.clone());
        let addr_0x_str_short = "0x".to_string() + &addr_str_short;
        let addr_0x_str_long = "0x".to_string() + &addr_str_long;

        assert!(Address::try_from(addr_vec_too_short).is_err());
        assert!(Address::try_from(addr_vec_too_long).is_err());

        assert!(Address::try_from(addr_str_short.clone()).is_err());
        assert!(Address::try_from(addr_str_long.clone()).is_err());

        assert!(Address::try_from(addr_0x_str_short.as_str()).is_err());
        assert!(Address::try_from(addr_0x_str_long.as_str()).is_err());

        let addr_vec_empty: Vec<u8> = vec![];
        let addr_str_empty = "".to_string();
        let addr_0x_str_empty = "0x".to_string();

        assert!(Address::try_from(addr_vec_empty).is_err());
        assert!(Address::try_from(addr_str_empty.as_str()).is_err());
        assert!(Address::try_from(addr_0x_str_empty.as_str()).is_err());
    }

    #[test]
    pub fn address_from_array() {
        let addr_arr: AddressArray = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x00, 0x11, 0x22, 0x33, 0x44,
        ];

        let test_addr = Address::test_create_address(&addr_arr.to_vec());

        assert_eq!(Address::from(addr_arr), test_addr);
        assert_eq!(Address::from(&addr_arr), test_addr);
    }

    #[test]
    pub fn address_to_string_vec() {
        let addr_vec: Vec<u8> = vec![
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x00, 0x11, 0x22, 0x33, 0x44,
        ];
        let address = Address::test_create_address(&addr_vec);

        assert_eq!(
            address.to_string(),
            "112233445566778899aabbccddeeff0011223344"
        );
        assert_eq!(address.to_vec(), addr_vec);
    }

    #[test]
    pub fn address_display() {
        let addr_vec: Vec<u8> = vec![
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x00, 0x11, 0x22, 0x33, 0x44,
        ];
        let address = Address::test_create_address(&addr_vec);

        assert_eq!(
            format!("{}", address),
            "112233445566778899aabbccddeeff0011223344"
        );
        assert_eq!(
            format!("{:?}", address),
            "112233445566778899aabbccddeeff0011223344"
        );
    }
}
