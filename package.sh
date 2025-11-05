wasm-pack build \
    --out-name wasmito-addr2line    \
    --target nodejs \
    --dev   \
    --no-opt    \
    --out-dir ../../pkg \
    ./crates/wasmito-addr2line-bindings
