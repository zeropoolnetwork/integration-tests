#!/usr/bin/env bash

echo "Building the client..."
yarn --cwd ./test-client build

echo "Serving the client..."
python3 -m http.server 3000 --directory ./test-client/dist &

echo "Running the selenium test..."
python3 selenium_test.py

kill $!
