use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AssetV0 {
    pub reports: Vec<Report>,
}

impl From<AssetV0> for Asset {
    fn from(v: AssetV0) -> Self {
        Asset {
            status: AssetStatus::Active,
            reports: v.reports,
            emas: vec![],
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AssetV1 {
    pub reports: Vec<Report>,
    pub emas: Vec<AssetEma>,
}

impl From<AssetV1> for Asset {
    fn from(v: AssetV1) -> Self {
        Asset {
            status: AssetStatus::Active,
            reports: v.reports,
            emas: v.emas,
        }
    }
}

