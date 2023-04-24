#!/bin/sh
# wasm-pack.exe build --target web --release

# build using rustc nightly 1.60 and some non-default flags
# taken from wasm-bindgen-rayon
# source: https://github.com/GoogleChromeLabs/wasm-bindgen-rayon#building-rust-code
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
 	rustup run nightly-2022-04-07 \
 	wasm-pack build --target web --release . \
 	-- -Z build-std=panic_abort,std
