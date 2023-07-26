# Upgrade through DAO

export CONTRACT_HASH="4sUttxKK4gJpWr1mhfNddBt497ZiXsRMbTYvCeQQbjR2"

near call $DAO_ACCOUNT_ID store_blob --base64 (base64 res/price_oracle.wasm) --accountId=$ACCOUNT_ID --amount=2.22 --gas=100000000000000

near call $DAO_ACCOUNT_ID add_proposal --accountId=$ACCOUNT_ID --amount=1 --gas=100000000000000 '{
  "proposal": {
    "description": "Upgrade to 0.6.0. Rework NEAR rebates",
    "kind": {
      "UpgradeRemote": {
        "receiver_id": "'$ORACLE_ID'",
        "method_name": "upgrade",
        "hash": "'$CONTRACT_HASH'"
      }
    }
  }
}'
