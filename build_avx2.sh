cargo +nightly --verbose rustc --features asm-comment --release -- -Z asm-comments -C "target-feature=+avx2" -C "llvm-args=-x86-asm-syntax=intel" --emit asm
