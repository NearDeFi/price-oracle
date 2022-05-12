use crate::*;

const MAX_F64_FOR_PRECISE_MULTIPLIER: f64 = 1e30;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetEma {
    pub period_sec: DurationSec,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub price: Option<Price>,
}

impl AssetEma {
    pub fn new(period_sec: DurationSec) -> AssetEma {
        Self {
            period_sec,
            timestamp: 0,
            price: None,
        }
    }

    pub fn recompute(&mut self, median_price: Price, timestamp: Timestamp) {
        if let Some(current) = self.price.as_mut() {
            let time_diff = timestamp - self.timestamp;
            // Based on https://stackoverflow.com/questions/1023860/exponential-moving-average-sampled-at-varying-times
            let alpha =
                1.0f64 - (-2.0f64 * time_diff as f64 / to_nano(self.period_sec) as f64).exp();
            let mut current_f64 = current.multiplier as f64;
            current_f64 *= 10f64.powi(median_price.decimals as i32 - current.decimals as i32);
            current_f64 += alpha * (median_price.multiplier as f64 - current_f64);
            if current_f64 <= MAX_F64_FOR_PRECISE_MULTIPLIER {
                *current = Price {
                    multiplier: (current_f64 * 1e4).round() as u128,
                    decimals: median_price.decimals + 4,
                }
            } else {
                *current = Price {
                    multiplier: current_f64.round() as u128,
                    decimals: median_price.decimals,
                }
            }
        } else {
            self.price = Some(median_price);
        }
        self.timestamp = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use crate::{to_nano, AssetEma, Price};
    use approx::assert_relative_eq;
    use near_sdk::Timestamp;

    fn ts(sec: u32) -> Timestamp {
        to_nano(1_600_000_000 + sec)
    }

    const BASE_DECIMALS: u8 = 28;

    fn mp(multiplier: u128) -> Price {
        Price {
            multiplier,
            decimals: BASE_DECIMALS,
        }
    }

    #[test]
    pub fn test_ema_init() {
        let mut ema = AssetEma {
            period_sec: 60000,
            timestamp: ts(0),
            price: None,
        };
        let timestamp = ts(10);
        let price = mp(100000);
        ema.recompute(price, timestamp);
        assert_eq!(ema.timestamp, timestamp);
        assert_eq!(ema.price, Some(price));
    }

    #[test]
    pub fn test_ema_period() {
        let price_multipliers = vec![
            22.27, 22.19, 22.08, 22.17, 22.18, 22.13, 22.23, 22.43, 22.24, 22.29, 22.15, 22.39,
            22.38, 22.61, 23.36, 24.05, 23.75, 23.83, 23.95, 23.63, 23.82, 23.87, 23.65, 23.19,
            23.10, 23.33, 22.68, 23.10, 22.40, 22.17,
        ];
        let expected_emas = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 22.22, 22.21, 22.24, 22.27, 22.33, 22.52,
            22.80, 22.97, 23.13, 23.28, 23.34, 23.43, 23.51, 23.53, 23.47, 23.40, 23.39, 23.26,
            23.23, 23.08, 22.92,
        ];
        let step = 60;
        let period_sec = step * 10;
        let mut ema = AssetEma {
            period_sec,
            timestamp: ts(0),
            price: None,
        };
        for (i, (multiplier, expected_ema)) in
            price_multipliers.into_iter().zip(expected_emas).enumerate()
        {
            let timestamp = ts(step * (i as u32 + 1));
            let price = mp((multiplier * 1e4) as u128);
            ema.recompute(price, timestamp);
            assert_eq!(ema.timestamp, timestamp);
            if expected_ema > 0.0 {
                let ema_price = ema.price.as_ref().unwrap();
                let ema_value = (ema_price.multiplier as f64)
                    / 10f64.powi(ema_price.decimals as i32 + 4 - BASE_DECIMALS as i32);
                assert_relative_eq!(ema_value, expected_ema, epsilon = 0.031);
            }
        }
    }
}
