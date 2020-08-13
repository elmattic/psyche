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
    use psyche::schedule::Schedule;
    use psyche::utils;
    use psyche::u256::U256;
    use psyche::vm::{VmMemory, VmRom, run_evm};

    const TEST_GAS: u64 = 1000;

    fn vm_assert_eq(input: &str, expected: &str, gas_limit: Option<U256>) {
        let schedule = Schedule::default();
        let gas_limit = gas_limit.unwrap_or(U256::from_u64(TEST_GAS));
        let bytes = assembler::from_string(input).unwrap();
        //
        let mut rom = VmRom::new();
        rom.init(&bytes, &schedule);
        let mut memory = VmMemory::new();
        memory.init(gas_limit);
        let word = unsafe {
            let ret_data = run_evm(&bytes, &rom, &schedule, gas_limit, &mut memory);
            memory.slice(ret_data.offset as isize, ret_data.size).to_vec()
        };
        let ref_word = utils::decode_hex(expected).unwrap();
        assert_eq!(word, ref_word);
    }

    #[test]
    fn opcode_add() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            ADD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None
        );
    }
}
