# psyche

> Fast Ethereum Virtual Machine implementation in Rust

## Project goals

1. Research, implement and optimize interpretation of EVM bytecode
   in stable Rust.
2. Explore around using special CPU instructions to speed up most opcodes.
3. Support a generic code path for best portability.
4. Optimize for the worst-case (do not optimize average-case at the expense of
   worst-case).
5. Hack **rustc** for better code generation for the ```loop```/```match```
   construct or guaranteed TCO.
6. Support the EVMC low-level ABI or integrate in Parity Ethereum.

## Build

Depending what you want to do you can either use:

- ```cargo build```
- ```cargo build --release```
- ```./build_debug.sh``` for an AVX2 build in debug
- ```./build_avx2.sh``` for inspecting assembly code for AVX2
  (see target/release/deps/psyche-xxx.s)
- ```./build_ssse3.sh``` for inspecting assembly code for SSSE3


Note that a build on macOS enables by default SSSE3 as those instructions are
always present on all Intel-based Macs.

## License

[LICENSE](https://github.com/elmattic/psyche/blob/master/LICENSE)
