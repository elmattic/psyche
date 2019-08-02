# psyche

> Fast Ethereum Virtual Machine implementation in Rust

## Project goals

- Research, implement and optimize interpretation of EVM bytecode
  in stable Rust.
- Explore around using special CPU instructions to speed up most opcodes.
- Support a generic code path for best portability.
- Optimize for the worst-case scenario (do not optimize the average-case if it
  is at the expense of the worst-case).
- Hack **rustc** for better code generation for the ```loop```/```match```
  construct or garanteed TCO (```become```).
- Integrate in Parity Ethereum client to see if it helps syncing.
- Support the EVMC low-level ABI.

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
