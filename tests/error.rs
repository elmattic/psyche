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

    fn vm_assert_eq(input: &str, expected: VmError, fork: Fork) {
        let schedule = Schedule::from_fork(fork);
        let gas_limit = U256::from_u64(TEST_GAS);
        let bytes = assembler::from_string(input).unwrap();
        //
        let mut rom = VmRom::new();
        rom.init(&bytes, &schedule);
        let mut memory = VmMemory::new();
        memory.init(gas_limit);
        let err = unsafe {
            let ret_data = run_evm(&bytes, &rom, &schedule, gas_limit, &mut memory);
            ret_data.error
        };
        assert_eq!(err, expected)
    }

    // Error on 'walk-into-subroutine'
    #[test]
    fn error_beginsub_0() {
        vm_assert_eq(
            "
            BEGINSUB
            ",
            VmError::BeginSubEntry,
            Fork::Berlin,
        );
    }

    // Invalid jump
    #[test]
    fn error_beginsub_1() {
        vm_assert_eq(
            "
            PUSH9 0x01000000000000000c
            JUMPSUB
            STOP
            BEGINSUB
            PUSH1 0x11
            JUMPSUB
            RETURNSUB
            BEGINSUB
            RETURNSUB
            ",
            VmError::InvalidBeginSub,
            Fork::Berlin,
        );
    }

    // Invalid jump inside a push
    #[test]
    fn error_beginsub_2() {
        vm_assert_eq(
            "
            PUSH1 0x04
            JUMPSUB
            PUSH2 0x003f ; 0x3f is BEGINSUB in VmRom and push bytes are swapped
            ",
            VmError::InvalidBeginSub,
            Fork::Berlin,
        );
    }

    // Return stack overflow
    #[test]
    fn error_jumpsub_0() {
        vm_assert_eq(
            "
            PUSH2 0x0015
            JUMP
            BEGINSUB
            DUP1
            ISZERO
            PUSH2 0x0013
            JUMPI
            PUSH1 0x01
            SWAP1
            SUB
            PUSH2 0x0004
            JUMPSUB
            JUMPDEST
            RETURNSUB
            JUMPDEST
            PUSH2 0x03ff
            PUSH2 0x0004
            JUMPSUB
            POP
            ",
            VmError::ReturnStackOverflow,
            Fork::Berlin
        );
    }

    // Error on invalid instruction
    #[test]
    fn error_jumpsub_1() {
        vm_assert_eq(
            "
            PUSH1 0x04
            JUMPSUB
            STOP
            BEGINSUB
            RETURNSUB
            ",
            VmError::InvalidInstruction,
            Fork::Istanbul,
        );
    }

    // Shallow return stack
    #[test]
    fn error_returnsub_0() {
        vm_assert_eq(
            "
            RETURNSUB
            ",
            VmError::ReturnStackUnderflow,
            Fork::Berlin,
        );
    }
}
