use crate::*;

pub type AssetId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Asset {
    pub status: AssetStatus,
    pub reports: Vec<Report>,
    pub emas: Vec<AssetEma>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum AssetStatus {
    Active,
    Hidden
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Report {
    pub oracle_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub price: Price,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetPrice {
    pub asset_id: AssetId,
    pub price: Price,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetOptionalPrice {
    pub asset_id: AssetId,
    pub price: Option<Price>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetOptionalValidatorPrice {
    pub asset_id: AssetId,
    pub price: Option<Price>,
    pub timestamp: Option<Timestamp>,
    pub status: Option<AssetStatus>
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAsset {
    V0(AssetV0),
    V1(AssetV1),
    Current(Asset),
}

impl From<VAsset> for Asset {
    fn from(v: VAsset) -> Self {
        match v {
            VAsset::V0(c) => c.into(),
            VAsset::V1(c) => c.into(),
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
            status: AssetStatus::Active,
            reports: Vec::new(),
            emas: Vec::new(),
        }
    }

    pub fn add_report(&mut self, report: Report) {
        self.reports.push(report);
    }

    pub fn remove_report(&mut self, oracle_id: &AccountId) -> bool {
        let initial_len = self.reports.len();
        self.reports.retain(|rp| &rp.oracle_id != oracle_id);
        self.reports.len() != initial_len
    }

    pub fn median_price(
        &self,
        timestamp_cut: Timestamp,
        min_num_recent_reports: usize,
    ) -> Option<Price> {
        let mut recent_reports: Vec<_> = self
            .reports
            .iter()
            .filter(|rp| rp.timestamp >= timestamp_cut)
            .collect();
        if recent_reports.len() < min_num_recent_reports {
            return None;
        }
        let index = recent_reports.len() / 2;
        recent_reports.select_nth_unstable_by(index, |a, b| a.price.cmp(&b.price));
        recent_reports.get(index).map(|tp| tp.price)
    }
}

impl Contract {
    pub fn internal_get_asset(&self, asset_id: &AssetId, use_status: bool) -> Option<Asset> {
        self.assets.get(asset_id).map(|v| {
            let asset: Asset = v.into();
            if use_status {
                match asset.status {
                    AssetStatus::Active => Some(asset),
                    AssetStatus::Hidden => None
                }
            }
            else {
                Some(asset)
            }
        }).unwrap_or(None)

    }

    pub fn internal_set_asset(&mut self, asset_id: &AssetId, asset: Asset) {
        self.assets.insert(asset_id, &asset.into());
    }

    pub fn internal_set_asset_status(&mut self, asset_id: AssetId, status: AssetStatus){
        if let Some(mut asset) = self.internal_get_asset(&asset_id, false) {
            asset.status = status;
            self.internal_set_asset(&asset_id, asset);
        }
        else {
            log!("Warning! Unknown asset ID: {}", asset_id);
        }
    }

    pub fn internal_get_asset_status(&self, asset_id: &AssetId) -> Option<AssetStatus>{
        if let Some(v_asset) =self.assets.get(asset_id) {
            Some(Asset::from(v_asset).status)
        }
        else {
            None
        }
    }
}
