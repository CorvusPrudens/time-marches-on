printf '[toolchain]\nchannel = "nightly-2025-06-01"' >rust-toolchain.toml
RUSTFLAGS="-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals" bevy build --no-default-features true --release \
  --bundle --locked \
  --features web-audio -Zbuild-std="std,core,alloc,panic_abort" web --wasm-opt false
rm rust-toolchain.toml
