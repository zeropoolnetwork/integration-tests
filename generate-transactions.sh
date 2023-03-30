#!/usr/bin/env bash

cd tx-generator
cargo run -- -m 'test test test test test test test test test test test junk' -n 100 -o ../txs.json
