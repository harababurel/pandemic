#!/bin/bash
set -xe

cargo build --target wasm32-unknown-unknown

wasm-bindgen --out-name pandemic \
             --out-dir target \
             --target web target/wasm32-unknown-unknown/debug/pandemic.wasm

basic-http-server -a 0.0.0.0:4000
