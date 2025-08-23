#!/bin/bash
curr="$PWD"
cd /home/kyle/coding/remarkable/rmpp-appload/examples/manhuagui || exit
# Only comment the import if it's not already commented
# sed -i '/^[[:space:]]*\/\/.*import xofm.libs.epaper as Epaper/!s|import xofm.libs.epaper as Epaper|//import xofm.libs.epaper as Epaper|' ui/example.qml

# # Only comment the block if it's not already commented
# sed -i '/Epaper\.ScreenModeItem {/,/}/{/^[[:space:]]*\/\//!s/^/\/\//}' ui/example.qml
rm -rf output
mkdir -p output/backend
cp icon.png manifest.json output
rcc --binary -o output/resources.rcc application.qrc
cd backend || exit
cargo build
cp target/debug/backend ../output/backend/entry
cd "$curr" || exit
