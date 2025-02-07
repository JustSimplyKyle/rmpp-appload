#!/bin/bash
rm -rf output-rmpp
mkdir -p output-rmpp/backend
cp icon.png manifest.json output-rmpp
rcc --binary -o output-rmpp/resources.rcc application.qrc
cd backend || exit
cargo build --target aarch64-unknown-linux-gnu --release
cp target/aarch64-unknown-linux-gnu/release/backend ../output-rmpp/backend/entry
cd ..

