#!/bin/bash
source /opt/codex/ferrari/5.0.58-dirty/environment-setup-cortexa53-crypto-remarkable-linux
sed -i 's|//import xofm.libs.epaper as Epaper|import xofm.libs.epaper as Epaper|' ui/example.qml 
sed -i '/\/\/.*Epaper\.ScreenModeItem {/,/\/\/.*}/s/^\/\///' ui/example.qml
rm -rf output-rmpp
mkdir -p output-rmpp/backend
cp icon.png manifest.json output-rmpp
rcc --binary -o output-rmpp/resources.rcc application.qrc
cd backend || exit
cargo zigbuild --target aarch64-unknown-linux-gnu.2.34 --release
cp target/aarch64-unknown-linux-gnu/release/backend ../output-rmpp/backend/entry
cd ..


