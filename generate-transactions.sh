#!/usr/bin/env bash

cd tx-generator
cargo run --release -- -m 'test test test test test test test test test test test junk' -n 10000 -o ../txs.json
