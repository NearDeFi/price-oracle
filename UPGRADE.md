# Upgrade through DAO

near call $DAO_ACCOUNT_ID store_blob --base64 (base64 res/price_oracle.wasm) --accountId=$ACCOUNT_ID --amount=2.2 --gas=100000000000000

export CONTRACT_HASH="Hvq9mdWGDBSvkbiaLoQRrP4KQC9bqjFueUMSh8BpEm3g"

near call $DAO_ACCOUNT_ID add_proposal --accountId=$ACCOUNT_ID --amount=1 --gas=100000000000000 '{
  "proposal": {
    "description": "Upgrade to 0.5.0. Compute asset EMAs",
    "kind": {
      "UpgradeRemote": {
        "receiver_id": "'$ORACLE_ID'",
        "method_name": "upgrade",
        "hash": "'$CONTRACT_HASH'"
      }
    }
  }
}'
