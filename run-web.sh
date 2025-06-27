printf '[toolchain]\nchannel = "nightly-2025-06-01"' >rust-toolchain.toml
RUSTFLAGS="-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals" bevy run --no-default-features true --release \
  --features web-audio -Zbuild-std="std,core,alloc,panic_abort" web --wasm-opt false \
  --headers "Cross-Origin-Opener-Policy: same-origin" \
  --headers "Cross-Origin-Embedder-Policy: require-corp"
rm rust-toolchain.toml
