use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AssetV0 {
    pub reports: Vec<Report>,
}

impl From<AssetV0> for Asset {
    fn from(v: AssetV0) -> Self {
        Asset {
            reports: v.reports,
            emas: vec![],
        }
    }
}
