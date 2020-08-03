cargo --verbose rustc --release -- -C target-feature=+avx2,+bmi2 -C llvm-args=-align-all-nofallthru-blocks=5
