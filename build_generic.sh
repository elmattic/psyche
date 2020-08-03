cargo +nightly --verbose rustc --features asm-comment --release -- -Z asm-comments -C target-feature=-ssse3 -C llvm-args=-align-all-nofallthru-blocks=5 -C llvm-args=-x86-asm-syntax=intel --emit asm
