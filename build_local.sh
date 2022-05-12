#!/bin/bash

perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' Cargo.toml

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/price_oracle.wasm ./res/

perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' Cargo.toml
