#!/usr/bin/env bash

export INFLUX_API_TOKEN="APL94vRX2Q5-_5oVaMPw2oHZzX2FzFaqV-7c9rhQmbtH9CXgbd79fVc9tyegUFLfDlF2gCEU36CePczokRFP4Q=="

echo "Building the client..."
yarn --cwd ./test-client build

echo "Serving the client..."
python3 -m http.server 3000 --directory ./test-client/dist &

cleanup() {
  kill %1
}

trap cleanup EXIT INT

echo "Running the selenium test..."
python3 tests/selenium_test.py

