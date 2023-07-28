use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{AccountId, Gas, Timestamp};
use near_sdk_sim::runtime::GenesisConfig;
use near_sdk_sim::{init_simulator, to_yocto, ExecutionResult, UserAccount};
use price_oracle::{AssetId, AssetPrice, DurationSec, Price, PriceData};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CONTARCT_WASM_BYTES => "res/price_oracle.wasm",
    CONTRACT_0_5_0_WASM_BYTES => "res/price_oracle_0.5.0.wasm",
}

const PREVIOUS_VERSION: &'static str = "0.5.0";
const LATEST_VERSION: &'static str = "0.6.0";

pub const DEFAULT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);

pub const NEAR: &str = "near";
pub const ORACLE_ID: &str = "oracle.near";
pub const OWNER_ID: &str = "owner.near";

pub const WRAP_NEAR: &str = "wrap.near";
pub const WRAP_NEAR_3600: &str = "wrap.near#3600";

pub fn to_nano(timestamp: u32) -> Timestamp {
    Timestamp::from(timestamp) * 10u64.pow(9)
}
pub fn ts(sec: u32) -> Timestamp {
    to_nano(1_600_000_000 + sec)
}
pub fn a(account_id: &str) -> AccountId {
    AccountId::new_unchecked(account_id.to_string())
}

pub struct Env {
    pub root: UserAccount,
    pub near: UserAccount,
    pub owner: UserAccount,
    pub contract: UserAccount,
    pub users: Vec<UserAccount>,
}

impl Env {
    pub fn setup(wasm_binary: &[u8]) -> Self {
        let mut genesis_config = GenesisConfig::default();
        genesis_config.genesis_time = ts(0);
        genesis_config.block_prod_time = 0;

        let root = init_simulator(Some(genesis_config));
        let near = root.create_user(a(NEAR), to_yocto("1000000"));
        let owner = near.create_user(a(OWNER_ID), to_yocto("10000"));

        let contract = near.deploy_and_init(
            wasm_binary,
            a(ORACLE_ID),
            "new",
            &json!({
                "recency_duration_sec": 90u32,
                "owner_id": OWNER_ID,
                "near_claim_amount": U128(to_yocto("5")),
            })
            .to_string()
            .into_bytes(),
            to_yocto("1000"),
            DEFAULT_GAS.0,
        );

        let users = (0..5)
            .map(|i| near.create_user(a(format!("user_{}.near", i).as_str()), to_yocto("100")))
            .collect();

        Self {
            root,
            near,
            owner,
            contract,
            users,
        }
    }

    pub fn skip_time(&self, seconds: u32) {
        self.near.borrow_runtime_mut().cur_block.block_timestamp += to_nano(seconds);
    }

    pub fn add_oracle(&self, user: &UserAccount) {
        self.owner
            .call(
                self.contract.account_id(),
                "add_oracle",
                &json!({
                    "account_id": user.account_id(),
                })
                .to_string()
                .into_bytes(),
                DEFAULT_GAS.0,
                1,
            )
            .assert_success();
    }

    pub fn add_asset(&self, asset_id: &str) {
        self.owner
            .call(
                self.contract.account_id(),
                "add_asset",
                &json!({
                    "asset_id": asset_id,
                })
                .to_string()
                .into_bytes(),
                DEFAULT_GAS.0,
                1,
            )
            .assert_success();
    }

    pub fn add_asset_ema(&self, asset_id: &str, period_sec: DurationSec) {
        self.owner
            .call(
                self.contract.account_id(),
                "add_asset_ema",
                &json!({
                    "asset_id": asset_id,
                    "period_sec": period_sec,
                })
                .to_string()
                .into_bytes(),
                DEFAULT_GAS.0,
                1,
            )
            .assert_success();
    }

    pub fn report_prices(&self, user: &UserAccount, prices: Vec<AssetPrice>) -> ExecutionResult {
        user.call(
            self.contract.account_id(),
            "report_prices",
            &json!({
                "prices": prices,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            0,
        )
    }

    pub fn get_price_data(&self, asset_ids: Option<Vec<AssetId>>) -> PriceData {
        self.near
            .view(
                self.contract.account_id(),
                "get_price_data",
                &json!({
                    "asset_ids": asset_ids,
                })
                .to_string()
                .into_bytes(),
            )
            .unwrap_json()
    }

    pub fn make_reports(&self, multipliers: &[u128]) {
        for (i, &multiplier) in multipliers.iter().enumerate() {
            if multiplier > 0 {
                self.report_prices(
                    &self.users[i],
                    vec![AssetPrice {
                        asset_id: WRAP_NEAR.to_string(),
                        price: Price {
                            multiplier,
                            decimals: 28,
                        },
                    }],
                )
                .assert_success();
            }
        }
    }
}

#[test]
pub fn test_init() {
    Env::setup(&CONTARCT_WASM_BYTES);
}

#[test]
pub fn test_basic() {
    let e = Env::setup(&CONTARCT_WASM_BYTES);

    e.add_oracle(&e.users[0]);
    e.add_oracle(&e.users[1]);
    e.add_oracle(&e.users[2]);

    e.add_asset(WRAP_NEAR);

    e.make_reports(&[100000, 110000, 106000]);

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.recency_duration_sec, 90);
    assert_eq!(price_data.timestamp, ts(0));
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );
}

#[test]
pub fn test_claim_near() {
    let e = Env::setup(&CONTARCT_WASM_BYTES);

    e.add_oracle(&e.users[0]);
    e.add_oracle(&e.users[1]);
    e.add_oracle(&e.users[2]);

    e.add_asset(WRAP_NEAR);

    let initial_balance = e.users[0].account().unwrap().amount;
    e.make_reports(&[100000, 110000, 106000]);
    let balance = e.users[0].account().unwrap().amount;
    // No refund by default
    assert!(initial_balance - balance < to_yocto("0.01"));

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );

    let initial_balance = balance;

    e.users[0]
        .call(
            e.contract.account_id(),
            "report_prices",
            &json!({
                "prices": vec![AssetPrice {
                    asset_id: WRAP_NEAR.to_string(),
                    price: Price {
                        multiplier: 108000,
                        decimals: 28,
                    },
                }],
                "claim_near": true,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            0,
        )
        .assert_success();

    let balance = e.users[0].account().unwrap().amount;
    // Received refund
    assert!(balance - initial_balance > to_yocto("4.99"));

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 108000,
            decimals: 28
        })
    );
}

#[test]
pub fn test_claim_near_no_oracle_balance() {
    let e = Env::setup(&CONTARCT_WASM_BYTES);

    e.add_oracle(&e.users[0]);
    e.add_oracle(&e.users[1]);
    e.add_oracle(&e.users[2]);

    e.add_asset(WRAP_NEAR);

    e.contract.transfer(
        e.users[0].account_id(),
        e.contract.account().unwrap().amount - to_yocto("6"),
    );
    let contract_balance = e.contract.account().unwrap().amount;
    assert!(contract_balance <= to_yocto("6"));

    let initial_balance = e.users[0].account().unwrap().amount;
    e.make_reports(&[100000, 110000, 106000]);
    let balance = e.users[0].account().unwrap().amount;
    // No refund by default
    assert!(initial_balance - balance < to_yocto("0.01"));

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );

    let initial_balance = balance;

    e.users[0]
        .call(
            e.contract.account_id(),
            "report_prices",
            &json!({
                "prices": vec![AssetPrice {
                    asset_id: WRAP_NEAR.to_string(),
                    price: Price {
                        multiplier: 108000,
                        decimals: 28,
                    },
                }],
                "claim_near": true,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            0,
        )
        .assert_success();

    let balance = e.users[0].account().unwrap().amount;
    // Still didn't receive the rebate, because there is not enough liquid balance.
    assert!(initial_balance - balance < to_yocto("0.01"));

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 108000,
            decimals: 28
        })
    );
}

#[test]
pub fn test_ema() {
    let e = Env::setup(&CONTARCT_WASM_BYTES);

    e.add_oracle(&e.users[0]);
    e.add_oracle(&e.users[1]);
    e.add_oracle(&e.users[2]);

    e.add_asset(WRAP_NEAR);

    e.make_reports(&[100000, 110000, 106000]);

    e.add_asset_ema(WRAP_NEAR, 3600);

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );

    let price_data = e.get_price_data(Some(vec![
        WRAP_NEAR.to_string(),
        WRAP_NEAR_3600.to_string(),
    ]));
    assert_eq!(price_data.prices.len(), 2);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );
    assert_eq!(&price_data.prices[1].asset_id, WRAP_NEAR_3600);
    assert!(price_data.prices[1].price.is_none());

    e.skip_time(60);

    e.make_reports(&[100000]);

    let price_data = e.get_price_data(Some(vec![
        WRAP_NEAR.to_string(),
        WRAP_NEAR_3600.to_string(),
    ]));
    assert_eq!(price_data.prices.len(), 2);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );
    assert_eq!(&price_data.prices[1].asset_id, WRAP_NEAR_3600);
    assert_eq!(
        &price_data.prices[1].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );

    e.make_reports(&[0, 110000, 106000]);

    let price_data = e.get_price_data(Some(vec![WRAP_NEAR_3600.to_string()]));
    assert_eq!(price_data.prices.len(), 1);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR_3600);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 106000,
            decimals: 28
        })
    );

    e.skip_time(60);

    e.make_reports(&[110000]);

    let price_data = e.get_price_data(Some(vec![
        WRAP_NEAR.to_string(),
        WRAP_NEAR_3600.to_string(),
    ]));
    assert_eq!(price_data.prices.len(), 2);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert_eq!(
        &price_data.prices[0].price,
        &Some(Price {
            multiplier: 110000,
            decimals: 28
        })
    );
    assert_eq!(&price_data.prices[1].asset_id, WRAP_NEAR_3600);
    assert_eq!(
        &price_data.prices[1].price,
        &Some(Price {
            multiplier: 1061311356,
            decimals: 32
        })
    );

    // Other 2 oracles didn't report within 2 minutes. Price shouldn't be available for WRAP_NEAR.
    e.skip_time(60);

    let price_data = e.get_price_data(Some(vec![
        WRAP_NEAR.to_string(),
        WRAP_NEAR_3600.to_string(),
    ]));
    assert_eq!(price_data.prices.len(), 2);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert!(price_data.prices[0].price.is_none());
    assert_eq!(&price_data.prices[1].asset_id, WRAP_NEAR_3600);
    assert_eq!(
        &price_data.prices[1].price,
        &Some(Price {
            multiplier: 1061311356,
            decimals: 32
        })
    );

    // In another minute, the EMA price becomes unavailable.
    e.skip_time(60);

    let price_data = e.get_price_data(Some(vec![
        WRAP_NEAR.to_string(),
        WRAP_NEAR_3600.to_string(),
    ]));
    assert_eq!(price_data.prices.len(), 2);
    assert_eq!(&price_data.prices[0].asset_id, WRAP_NEAR);
    assert!(price_data.prices[0].price.is_none());
    assert_eq!(&price_data.prices[1].asset_id, WRAP_NEAR_3600);
    assert!(price_data.prices[1].price.is_none());
}

#[test]
pub fn test_update() {
    let e = Env::setup(&CONTRACT_0_5_0_WASM_BYTES);

    e.add_oracle(&e.users[0]);
    e.add_oracle(&e.users[1]);
    e.add_oracle(&e.users[2]);

    e.add_asset(WRAP_NEAR);

    e.make_reports(&[100000, 110000, 106000]);

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);

    let version: String = e
        .near
        .view(e.contract.account_id(), "get_version", &[])
        .unwrap_json();

    assert_eq!(version, PREVIOUS_VERSION);

    e.owner
        .create_transaction(a(ORACLE_ID))
        .function_call(
            "upgrade".to_string(),
            CONTARCT_WASM_BYTES.to_vec(),
            MAX_GAS.0,
            0,
        )
        .submit();

    let price_data = e.get_price_data(None);
    assert_eq!(price_data.prices.len(), 1);

    let version: String = e
        .near
        .view(e.contract.account_id(), "get_version", &[])
        .unwrap_json();

    assert_eq!(version, LATEST_VERSION);
}

#[test]
fn test_version() {
    let e = Env::setup(&CONTARCT_WASM_BYTES);

    let version: String = e
        .near
        .view(e.contract.account_id(), "get_version", &[])
        .unwrap_json();

    assert_eq!(version, LATEST_VERSION);
}
