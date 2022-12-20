use near_sdk::require;
use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn hide_asset(&mut self, asset_id: AssetId){
        assert_one_yocto();
        require!(self.internal_get_oracle(&env::predecessor_account_id()).is_some(), "Not an oracle validator");
        self.internal_set_asset_status(asset_id, AssetStatus::Hidden);
    }
}