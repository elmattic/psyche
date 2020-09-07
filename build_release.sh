cargo --verbose rustc --lib --release -- -C target-feature=+avx2,+bmi2,+lzcnt -C llvm-args=-align-all-nofallthru-blocks=5
