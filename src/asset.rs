use crate::*;
use near_sdk::Duration;
use std::collections::HashMap;

pub type AssetId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Asset {
    /// A map of prices from oracle account ID to a TimedPrice.
    pub prices: HashMap<AccountId, TimedPrice>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TimedPrice {
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub price: Fraction,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetPrice {
    pub asset_id: AssetId,
    pub price: Fraction,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetOptionalPrice {
    pub asset_id: AssetId,
    pub price: Option<Fraction>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAsset {
    Current(Asset),
}

impl From<VAsset> for Asset {
    fn from(v: VAsset) -> Self {
        match v {
            VAsset::Current(c) => c,
        }
    }
}

impl From<Asset> for VAsset {
    fn from(c: Asset) -> Self {
        VAsset::Current(c)
    }
}

impl Asset {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
        }
    }

    pub fn median_price(&self, recency_duration: Duration) -> Option<Fraction> {
        let timestamp_cut = env::block_timestamp().saturating_sub(recency_duration);
        let mut recent_prices: Vec<_> = self
            .prices
            .values()
            .filter(|tp| tp.timestamp >= timestamp_cut)
            .collect();
        if recent_prices.is_empty() {
            return None;
        }
        let index = recent_prices.len() / 2;
        recent_prices.select_nth_unstable_by(index, |a, b| a.price.cmp(&b.price));
        recent_prices.get(index).map(|tp| tp.price)
    }
}

impl Contract {
    pub fn internal_get_asset(&self, asset_id: &AssetId) -> Option<Asset> {
        self.assets.get(asset_id).map(|v| v.into())
    }

    pub fn internal_set_asset(&mut self, asset_id: &AssetId, asset: Asset) {
        self.assets.insert(asset_id, &asset.into());
    }
}
