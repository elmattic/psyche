// Copyright 2020 The Psyche Authors
// This file is part of Psyche.
//
// Psyche is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Psyche is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Psyche. If not, see <http://www.gnu.org/licenses/>.

#[cfg(test)]
mod tests {
    use psyche::assembler;
    use psyche::schedule::{Fork, Schedule};
    use psyche::u256::U256;
    use psyche::vm::{run_evm, VmError, VmMemory, VmRom};

    const TEST_GAS: u64 = 20_000_000_000_000;

    fn vm_assert_eq(input: &str, expected: Result<u64, VmError>, fork: Fork) {
        let schedule = Schedule::from_fork(fork);
        let gas_limit = U256::from_u64(TEST_GAS);
        let bytes = assembler::from_string(input).unwrap();
        //
        let mut rom = VmRom::new();
        rom.init(&bytes, &schedule);
        let mut memory = VmMemory::new();
        memory.init(gas_limit);
        let (gas_used, err) = unsafe {
            let ret_data = run_evm(&bytes, &rom, &schedule, gas_limit, &mut memory);
            (TEST_GAS.wrapping_sub(ret_data.gas), ret_data.error)
        };
        if err == VmError::None {
            assert_eq!(Ok(gas_used), expected);
        } else {
            assert_eq!(Err(err), expected)
        }
    }

    #[test]
    fn gas_sha3_0() {
        vm_assert_eq(
            "
            PUSH1 0x00
            PUSH1 0x00
            SHA3
            ",
            Ok(3 + 3 + (30 + (0 * 6)) + (0 + (3 * 0))),
            Fork::default(),
        );
    }

    #[test]
    fn gas_sha3_1() {
        vm_assert_eq(
            "
            PUSH1 0x01
            PUSH1 0x00
            SHA3
            ",
            Ok(3 + 3 + (30 + (1 * 6)) + (0 + (3 * 1))),
            Fork::default(),
        );
    }

    #[test]
    fn gas_sha3_2() {
        vm_assert_eq(
            "
            PUSH1 0x21
            PUSH1 0x00
            SHA3
            ",
            Ok(3 + 3 + (30 + (2 * 6)) + (0 + (3 * 2))),
            Fork::default(),
        );
    }

    #[test]
    fn gas_sha3_3() {
        vm_assert_eq(
            "
            PUSH2 0x02e0
            PUSH1 0x00
            SHA3
            ",
            Ok(3 + 3 + (30 + (23 * 6)) + (1 + (3 * 23))),
            Fork::default(),
        );
    }

    #[test]
    fn gas_sha3_4() {
        vm_assert_eq(
            "
            PUSH8 0x3fffffffffffffff
            PUSH1 0x00
            SHA3
            ",
            Err(VmError::OutOfGas),
            Fork::default(),
        );
    }
}
