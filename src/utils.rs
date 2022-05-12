use crate::*;
use std::cmp::Ordering;

const MAX_U128_DECIMALS: u8 = 38;
const MAX_VALID_DECIMALS: u8 = 77;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    #[serde(with = "u128_dec_format")]
    pub multiplier: Balance,
    pub decimals: u8,
}

// 5 NEAR = 5 * 10**24 "wrap.near"
// 50 DAI = 50 * 10**18 "dai.bridge.near"

// Price NEAR { multiplier: 1000, decimals: 26 }
// 5 NEAR in USD = 5 * 10**24 * 1000 / 10**(26 - 18) = 50 * 10**18
// Price DAI { multiplier: 101, decimals: 20 }
// 50 DAI in USD = 50 * 10**18 * 101 / 10**(20 - 18) = 505 * 10**17

impl Price {
    pub fn assert_valid(&self) {
        assert!(self.decimals <= MAX_VALID_DECIMALS);
    }
}

impl PartialEq<Self> for Price {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.decimals < other.decimals {
            return Some(other.cmp(self).reverse());
        }

        let decimals_diff = self.decimals - other.decimals;

        if decimals_diff > MAX_U128_DECIMALS {
            return Some(Ordering::Less);
        }

        if let Some(om) = other
            .multiplier
            .checked_mul(10u128.pow(decimals_diff as u32))
        {
            Some(self.multiplier.cmp(&om))
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Eq for Price {}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub(crate) mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub(crate) mod u64_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub(crate) fn to_nano(ts: u32) -> Timestamp {
    Timestamp::from(ts) * 10u64.pow(9)
}

pub(crate) fn unordered_map_pagination<K, VV, V>(
    m: &UnorderedMap<K, VV>,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<(K, V)>
where
    K: BorshSerialize + BorshDeserialize,
    VV: BorshSerialize + BorshDeserialize,
    V: From<VV>,
{
    let keys = m.keys_as_vector();
    let values = m.values_as_vector();
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(keys.len());
    (from_index..std::cmp::min(keys.len(), from_index + limit))
        .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap().into()))
        .collect()
}
