#!/bin/sh

set -ex

cargo build --target wasm32-unknown-unknown --release
cd ../../
rm -rf static
mkdir static
cp target/wasm32-unknown-unknown/release/rusty_demon_attack.wasm static/
cp utils/wasm/index.html static/
cp utils/wasm/gl.js static/
cp utils/wasm/audio.js static/
mkdir static/resources
cp -ar resources static/
ls -lh static