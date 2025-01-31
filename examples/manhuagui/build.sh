#!/bin/bash
curr="$PWD"
cd /home/kyle/coding/remarkable/rmpp-appload/examples/manhuagui || exit
rm -rf output
mkdir -p output/backend
cp icon.png manifest.json output
rcc --binary -o output/resources.rcc application.qrc
cd backend || exit
cargo build
cp target/debug/backend ../output/backend/entry
cd "$curr" || exit
