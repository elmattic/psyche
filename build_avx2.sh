cargo +nightly --verbose rustc --lib --features asm-comment --release -- -Z asm-comments -C target-feature=+avx2,+bmi2 -C llvm-args=-align-all-nofallthru-blocks=5 -C llvm-args=-x86-asm-syntax=intel --emit asm
