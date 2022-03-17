use crate::*;
use near_sdk::json_types::U128;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_recency_duration_sec(&mut self, recency_duration_sec: DurationSec) {
        assert_one_yocto();
        self.assert_owner();
        self.recency_duration_sec = recency_duration_sec;
    }

    #[payable]
    pub fn add_oracle(&mut self, account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert!(self.internal_get_oracle(&account_id).is_none());
        self.internal_set_oracle(&account_id, Oracle::new());
    }

    #[payable]
    pub fn remove_oracle(&mut self, account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert!(self.oracles.remove(&account_id).is_some());
    }

    #[payable]
    pub fn add_asset(&mut self, asset_id: AssetId) {
        assert_one_yocto();
        self.assert_owner();
        assert!(self.internal_get_asset(&asset_id).is_none());
        self.internal_set_asset(&asset_id, Asset::new());
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn get_near_claim_amount(&self) -> U128 {
        self.near_claim_amount.into()
    }

    #[payable]
    pub fn update_near_claim_amount(&mut self, near_claim_amount: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.near_claim_amount = near_claim_amount.into();
    }

    #[payable]
    pub fn update_owner_id(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.owner_id = owner_id;
    }
}

impl Contract {
    pub fn assert_owner(&self) {
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "Can only be called by the owner"
        );
    }
}
