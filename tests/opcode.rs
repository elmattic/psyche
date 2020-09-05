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

    const TEST_GAS: u64 = 20_000_000_000_000;

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
    fn opcode_add_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            ADD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_add_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            ADD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_add_2() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            ADD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mul_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            MUL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mul_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            MUL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mul_2() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH1 0x01
            MUL
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_mul_3() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH17 0x0100000000000000000000000000000000
            MUL
            retword
            ",
            "c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d700000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sub_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            SUB
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sub_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SUB
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sub_2() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SUB
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sub_3() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            SUB
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sub_4() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0
            SUB
            retword
            ",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1",
            None,
        );
    }

    #[test]
    fn opcode_div_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            DIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_div_1() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x01
            DIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_div_2() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x80
            DIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000040",
            None,
        );
    }

    #[test]
    fn opcode_div_3() {
        vm_assert_eq("
            PUSH8  0x58d2c9fd8890dca1
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            DIV
            retword
            ",
            "0000000000000001023c7d42ce0a384892d4df16d68433e7185bc4008a5901d5",
            None,
        );
    }

    #[test]
    fn opcode_div_4() {
        vm_assert_eq("
            PUSH4  0x58d2c9fd
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            DIV
            retword
            ",
            "00000001023c7d445b13f21c9695cd98f93bb2ff1fcb0ef8913c8735747f88f8",
            None,
        );
    }

    #[test]
    fn opcode_mod_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            MOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mod_1() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x01
            MOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_mod_2() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x80
            MOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mod_3() {
        vm_assert_eq("
            PUSH8  0x58d2c9fd8890dca1
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            MOD
            retword
            ",
            "0000000000000000000000000000000000000000000000004bcb1d3950a2cd0b",
            None,
        );
    }

    #[test]
    fn opcode_mod_4() {
        vm_assert_eq("
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            PUSH8  0x58d2c9fd8890dca1
            MOD
            retword
            ",
            "00000000000000000000000000000000000000000000000058d2c9fd8890dca1",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SDIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_1() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x01
            SDIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_2() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x80
            SDIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000040",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_3() {
        vm_assert_eq("
            PUSH8  0x58d2c9fd8890dca1
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            SDIV
            retword
            ",
            "0000000000000001023c7d42ce0a384892d4df16d68433e7185bc4008a5901d5",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_4() {
        vm_assert_eq("
            PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe
            PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc
            SDIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000002",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_5() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000002
            PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc
            SDIV
            retword
            ",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_6() {
        vm_assert_eq("
            PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000004
            SDIV
            retword
            ",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
            None,
        );
    }

    #[test]
    fn opcode_sdiv_7() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000002
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000004
            SDIV
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000002",
            None,
        );
    }

    #[test]
    fn opcode_smod_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_smod_1() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x01
            SMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_smod_2() {
        vm_assert_eq("
            PUSH1 0x02
            PUSH1 0x80
            SMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_smod_3() {
        vm_assert_eq("
            PUSH8  0x58d2c9fd8890dca1
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            SMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000004bcb1d3950a2cd0b",
            None,
        );
    }

    #[test]
    fn opcode_smod_4() {
        vm_assert_eq("
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            PUSH8  0x58d2c9fd8890dca1
            SMOD
            retword
            ",
            "00000000000000000000000000000000000000000000000058d2c9fd8890dca1",
            None,
        );
    }

    #[test]
    fn opcode_smod_5() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000003
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000004
            SMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_smod_6() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000003
            PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc
            SMOD
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    /*#[test]
    fn opcode_addmod_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            PUSH1 0x01
            ADDMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_addmod_1() {
        vm_assert_eq("
            PUSH8 0xffffffffffffffff
            PUSH8 0xfffffffffffffff0
            PUSH8 0x000000000000000f
            ADDMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_addmod_2() {
        vm_assert_eq("
            PUSH8 0xffffffffffffffff
            PUSH8 0xfffffffffffffff0
            PUSH8 0x0000000000000000
            ADDMOD
            retword
            ",
            "000000000000000000000000000000000000000000000000fffffffffffffff0",
            None,
        );
    }

    #[test]
    fn opcode_addmod_3() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            ADDMOD
            retword
            ",
            "1e1b1815120f0c08edeae7e4e1dedbd8bdbab7b4b1aeaba88d8a8784817e7b79",
            None,
        );
    }

    #[test]
    fn opcode_mulmod_0() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1 0x00
            PUSH1 0x00
            MULMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mulmod_1() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1 0x00
            PUSH1 0x01
            MULMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mulmod_2() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH1 0x01
            MULMOD
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_mulmod_3() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH32 0x00000000000000000000000000000000a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7
            PUSH17 0x0100000000000000000000000000000000
            MULMOD
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b700000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mulmod_4() {
        vm_assert_eq("
            PUSH32 0x59996c6ef58409a71b05be0bada2445eb7c017d09442e7d158e0000000000000
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            MULMOD
            retword
            ",
            "2e10b923f6e797bc6f027b58fab1f793ea38565feab51d529bae2d2c2b2a2929",
            None,
        );
    }

    #[test]
    fn opcode_mulmod_5() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            MULMOD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_exp_0() {
        vm_assert_eq("
            0
            2
            EXP
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_exp_1() {
        vm_assert_eq("
            1
            2
            EXP
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000002",
            None,
        );
    }

    #[test]
    fn opcode_exp_2() {
        vm_assert_eq("
            255
            2
            EXP
            retword
            ",
            "8000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_exp_3() {
        vm_assert_eq("
            18446744073709551616
            3
            EXP
            retword
            ",
            "c2ee4df12b16bb31d6c4c9537a102fceaac77ae32292e8f40000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_exp_4() {
        vm_assert_eq("
            340282366920938463463374607431768211456
            3
            EXP
            retword
            ",
            "8b7a43dfccea9b86aac77ae32292e8f400000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_exp_5() {
        vm_assert_eq("
            6277101735386680763835789423207666416102355444464034512896
            3
            EXP
            retword
            ",
            "aac77ae32292e8f4000000000000000000000000000000000000000000000001",
            None,
        );
    }*/

    #[test]
    fn opcode_signextend_0() {
        vm_assert_eq("
            PUSH32 0x0000faffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            29
            SIGNEXTEND
            retword
            ",
            "fffffaffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_signextend_1() {
        vm_assert_eq("
            PUSH32 0x0000faffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            28
            SIGNEXTEND
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_signextend_2() {
        vm_assert_eq("
            PUSH32 0x00000000000000000000000000000000000000000000000000000000000000fa
            0
            SIGNEXTEND
            retword
            ",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa",
            None,
        );
    }

    #[test]
    fn opcode_signextend_3() {
        vm_assert_eq("
            PUSH32 0x00007affffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            29
            SIGNEXTEND
            retword
            ",
            "00007affffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_signextend_4() {
        vm_assert_eq("
            PUSH32 0x0000faffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            32
            SIGNEXTEND
            retword
            ",
            "0000faffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_signextend_5() {
        vm_assert_eq("
            PUSH32 0x0000faffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            340282366920938463463374607431768211456
            SIGNEXTEND
            retword
            ",
            "0000faffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_lt_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_lt_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_lt_2() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_lt_3() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_lt_4() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_lt_5() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_lt_6() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            LT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_gt_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_gt_1() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_gt_2() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_gt_3() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_gt_4() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_gt_5() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_gt_6() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            GT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_slt_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_slt_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_slt_2() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_slt_3() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_slt_4() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_slt_5() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_slt_6() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_slt_7() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_slt_8() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffff00000000000000000000000000000001
            PUSH32 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000
            SLT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sgt_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sgt_1() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sgt_2() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sgt_3() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sgt_4() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sgt_5() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000000
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sgt_6() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000100000000000000000000000000000001
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sgt_7() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sgt_8() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000
            PUSH32 0xffffffffffffffffffffffffffffffff00000000000000000000000000000001
            SGT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_eq_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            EQ
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_eq_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            EQ
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_eq_2() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            EQ
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_iszero_0() {
        vm_assert_eq("
            PUSH1 0x00
            ISZERO
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_iszero_1() {
        vm_assert_eq("
            PUSH1 0x01
            ISZERO
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_and_0() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            AND
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_and_1() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            AND
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_or_0() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            OR
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_or_1() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            OR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_xor_0() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            XOR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_xor_1() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            XOR
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_not_0() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            NOT
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_not_1() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            NOT
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_byte_0() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            0
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000a0",
            None,
        );
    }

    #[test]
    fn opcode_byte_1() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            7
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000a7",
            None,
        );
    }

    #[test]
    fn opcode_byte_2() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            8
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000b0",
            None,
        );
    }

    #[test]
    fn opcode_byte_3() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            15
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000b7",
            None,
        );
    }

    #[test]
    fn opcode_byte_4() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            16
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000c0",
            None,
        );
    }

    #[test]
    fn opcode_byte_5() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            23
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000c7",
            None,
        );
    }

    #[test]
    fn opcode_byte_6() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            24
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000d0",
            None,
        );
    }

    #[test]
    fn opcode_byte_7() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            31
            BYTE
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000d7",
            None,
        );
    }

    #[test]
    fn opcode_byte_8() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            32
            BYTE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_byte_9() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            18446744073709551616
            BYTE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_byte_10() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            340282366920938463463374607431768211456
            BYTE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_0() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0x00
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_shl_1() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0x01
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000002",
            None,
        );
    }

    #[test]
    fn opcode_shl_2() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0xff
            SHL
            retword
            ",
            "8000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_3() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH2  0x0100
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_4() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH2  0x0101
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_5() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x00
            SHL
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_shl_6() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x01
            SHL
            retword
            ",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
            None,
        );
    }

    #[test]
    fn opcode_shl_7() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0xff
            SHL
            retword
            ",
            "8000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_8() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH2  0x0100
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_9() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0x01
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_10() {
        vm_assert_eq("
            PUSH32 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x01
            SHL
            retword
            ",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
            None,
        );
    }

    #[test]
    fn opcode_shl_11() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH17 0x0100000000000000000000000000000000
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_12() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH25 0x02000000000000000000000000000000000000000000000000
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_13() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH32 0xffffffffffffffff000000000000000000000000000000000000000000000000
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_14() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH32 0x000000000000000000000000000000000000000000000000ffffffffffffffff
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_15() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000000000000000000000000000100000001
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_16() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            64
            SHL
            retword
            ",
            "0000000000000000000000000000000000000000000000010000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_17() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            1
            SHL
            retword
            ",
            "41434547494b4d4f61636567696b6d6f81838587898b8d8fa1a3a5a7a9abadae",
            None,
        );
    }

    #[test]
    fn opcode_shl_18() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            136
            SHL
            retword
            ",
            "c1c2c3c4c5c6c7d0d1d2d3d4d5d6d70000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shl_19() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            248
            SHL
            retword
            ",
            "d700000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shr_0() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0x00
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_shr_1() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0x01
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shr_2() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0x01
            SHR
            retword
            ",
            "4000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shr_3() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0xff
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_shr_4() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH2  0x0100
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shr_5() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH2  0x0101
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shr_6() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x00
            SHR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_shr_7() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x01
            SHR
            retword
            ",
            "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_shr_8() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0xff
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_shr_9() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH2  0x0100
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_shr_10() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0x01
            SHR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sar_0() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0x00
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sar_1() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH1  0x01
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sar_2() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0x01
            SAR
            retword
            ",
            "c000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sar_3() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0xff
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_4() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH2  0x0100
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_5() {
        vm_assert_eq("
            PUSH32 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH2  0x0101
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_6() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x00
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_7() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0x01
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_8() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0xff
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_9() {
        vm_assert_eq("
            PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH2  0x0100
            SAR
            retword
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_sar_10() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0x01
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sar_11() {
        vm_assert_eq("
            PUSH32 0x4000000000000000000000000000000000000000000000000000000000000000
            PUSH1  0xfe
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sar_12() {
        vm_assert_eq("
            PUSH32 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0xf8
            SAR
            retword
            ",
            "000000000000000000000000000000000000000000000000000000000000007f",
            None,
        );
    }

    #[test]
    fn opcode_sar_13() {
        vm_assert_eq("
            PUSH32 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0xfe
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_sar_14() {
        vm_assert_eq("
            PUSH32 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH1  0xff
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sar_15() {
        vm_assert_eq("
            PUSH32 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH2  0x0100
            SAR
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    /*#[test]
    fn opcode_sha3_0() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            SHA3
            retword
            ",
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
            None,
        );
    }

    #[test]
    fn opcode_sha3_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SHA3
            retword
            ",
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
            None,
        );
    }

    #[test]
    fn opcode_sha3_2() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SHA3
            retword
            ",
            "bc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a",
            None,
        );
    }

    #[test]
    fn opcode_sha3_3() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            MSTORE
            PUSH1 0x01
            PUSH1 0x00
            SHA3
            retword
            ",
            "bc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a",
            None,
        );
    }

    #[test]
    fn opcode_sha3_4() {
        vm_assert_eq("
            PUSH32 0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3
            PUSH1  0x00
            MSTORE
            PUSH1 0x20
            PUSH1 0x00
            SHA3
            retword
            ",
            "456f7311d51b58bd3194db04d5507395f9fc99188c6ac92829a0402e2f130d53",
            None,
        );
    }

    #[test]
    fn opcode_sha3_5() {
        vm_assert_eq("
            PUSH32 0x00d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8f
            PUSH1  0x00
            MSTORE
            PUSH32 0xa300000000000000000000000000000000000000000000000000000000000000
            PUSH1  0x20
            MSTORE
            PUSH1 0x20
            PUSH1 0x01
            SHA3
            retword
            ",
            "456f7311d51b58bd3194db04d5507395f9fc99188c6ac92829a0402e2f130d53",
            None,
        );
    }

    #[test]
    fn opcode_sha3_6() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            SHA3
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sha3_7() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x01
            SHA3
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_sha3_8() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SHA3
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000020",
            None,
        );
    }

    #[test]
    fn opcode_sha3_9() {
        vm_assert_eq("
            PUSH1 0x21
            PUSH1 0x00
            SHA3
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000040",
            None,
        );
    }

    #[test]
    fn opcode_sha3_10() {
        vm_assert_eq("
            PUSH1 0x20
            PUSH1 0x01
            SHA3
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000040",
            None,
        );
    }

    #[test]
    fn opcode_caller_0() {
        vm_assert_eq("
            CALLER
            retword
            ",
            "000000000000000000000000000000000000000000000000000073656e646572",
            None,
        );
    }

    #[test]
    fn opcode_codesize_0() {
        vm_assert_eq("
            CODESIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000009",
            None,
        );
    }

    #[test]
    fn opcode_codesize_1() {
        vm_assert_eq("
            PUSH1 0x00
            CODESIZE
            retword
            ",
            "000000000000000000000000000000000000000000000000000000000000000b",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_0() {
        vm_assert_eq("
            JUMPDEST
            JUMPDEST
            PUSH2 0x0008
            PUSH1 0x02
            PUSH1 0x00
            CODECOPY
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "6100086002600039000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_1() {
        vm_assert_eq("
            JUMPDEST
            JUMPDEST
            PUSH2 0x0008
            PUSH1 0x02
            PUSH1 0x00
            CODECOPY
            MSIZE
            PUSH1 0x00
            MSTORE
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "0000000000000000000000000000000000000000000000000000000000000020",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_2() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SUB
            PUSH1 0x00
            MSTORE
            JUMPDEST
            JUMPDEST
            PUSH2 0x0008
            PUSH1 0x0a
            PUSH1 0x00
            CODECOPY
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "610008600a600039ffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_3() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SUB
            PUSH1 0x00
            MSTORE
            JUMPDEST
            JUMPDEST
            PUSH2 0x0008
            PUSH1 0x0a
            PUSH1 0x02
            CODECOPY
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "ffff610008600a600239ffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_4() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SUB
            PUSH1 0x00
            MSTORE
            JUMPDEST
            JUMPDEST
            PUSH2 0x0017
            PUSH1 0x0a
            PUSH1 0x00
            CODECOPY
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "610017600a60003960206000f300000000000000000000ffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_5() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SUB
            PUSH1 0x00
            MSTORE
            JUMPDEST
            JUMPDEST
            PUSH2 0x0017
            PUSH1 0xff
            PUSH1 0x00
            CODECOPY
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "0000000000000000000000000000000000000000000000ffffffffffffffffff",
            None,
        );
    }

    #[test]
    fn opcode_codecopy_6() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            SUB
            PUSH1 0x00
            MSTORE
            JUMPDEST
            JUMPDEST
            PUSH2 0x0000
            PUSH1 0x0a
            PUSH1 0x00
            CODECOPY
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            None,
        );
    }*/

    #[test]
    fn opcode_pop_0() {
        vm_assert_eq("
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH32 0x0000000000000000000000000000000000000000000000000000000000000002
            POP
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_mload_0() {
        vm_assert_eq("
            PUSH1 0x00
            MLOAD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mload_1() {
        vm_assert_eq("
            PUSH1 0x01
            MLOAD
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_mstore_0() {
        vm_assert_eq("
            PUSH1 0xff
            0
            MSTORE
            0
            MLOAD
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000ff",
            None,
        );
    }

    #[test]
    fn opcode_mstore_1() {
        vm_assert_eq("
            PUSH1 0xff
            1
            MSTORE
            1
            MLOAD
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000ff",
            None,
        );
    }

    #[test]
    fn opcode_mstore_2() {
        vm_assert_eq("
            PUSH1 0xff
            0
            MSTORE
            0
            MLOAD
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000ff",
            None,
        );
    }

    #[test]
    fn opcode_mstore_3() {
        vm_assert_eq("
            PUSH1 0xff
            1
            MSTORE
            1
            MLOAD
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000ff",
            None,
        );
    }

    #[test]
    fn opcode_jump_0() {
        vm_assert_eq("
            PUSH1 0x04
            JUMP
            STOP
            JUMPDEST
            0
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_jumpi_0() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x06
            JUMPI
            STOP
            JUMPDEST
            PUSH1 0x00
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_jumpi_1() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0xff
            JUMPI
            PUSH1 0x00
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_pc_0() {
        vm_assert_eq("
            PC
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_pc_1() {
        vm_assert_eq("
            PUSH1 0x00
            PC
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000002",
            None,
        );
    }

    #[test]
    fn opcode_msize_0() {
        vm_assert_eq("
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_msize_1() {
        vm_assert_eq("
            PUSH1 0x00
            MLOAD
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000020",
            None,
        );
    }

    #[test]
    fn opcode_msize_2() {
        vm_assert_eq("
            PUSH1 0x00
            PUSH1 0x00
            MSTORE
            MSIZE
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000000020",
            None,
        );
    }

    #[test]
    fn opcode_gas_0() {
        vm_assert_eq("
            GAS
            retword
            ",
            "000000000000000000000000000000000000000000000000000012309ce53ffe",
            None,
        );
    }

    #[test]
    fn opcode_push1_0() {
        vm_assert_eq("
            PUSH1 0xd7
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000000000d7",
            None,
        );
    }

    #[test]
    fn opcode_push2_0() {
        vm_assert_eq("
            PUSH2 0xd6d7
            retword
            ",
            "000000000000000000000000000000000000000000000000000000000000d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push4_0() {
        vm_assert_eq("
            PUSH4 0xd4d5d6d7
            retword
            ",
            "00000000000000000000000000000000000000000000000000000000d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push3_0() {
        vm_assert_eq("
            PUSH3 0xd5d6d7
            retword
            ",
            "0000000000000000000000000000000000000000000000000000000000d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push5_0() {
        vm_assert_eq("
            PUSH5 0xd3d4d5d6d7
            retword
            ",
            "000000000000000000000000000000000000000000000000000000d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push6_0() {
        vm_assert_eq("
            PUSH6 0xd2d3d4d5d6d7
            retword
            ",
            "0000000000000000000000000000000000000000000000000000d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push7_0() {
        vm_assert_eq("
            PUSH7 0xd1d2d3d4d5d6d7
            retword
            ",
            "00000000000000000000000000000000000000000000000000d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push8_0() {
        vm_assert_eq("
            PUSH8 0xd0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000000000000000000000000000000000000000d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push9_0() {
        vm_assert_eq("
            PUSH9 0xc7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000000000000000000000000000000000000000c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push10_0() {
        vm_assert_eq("
            PUSH10 0xc6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000000000000000000000000000000000000000c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push11_0() {
        vm_assert_eq("
            PUSH11 0xc5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000000000000000000000000000000000c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push12_0() {
        vm_assert_eq("
            PUSH12 0xc4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000000000000000000000000000000000c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push13_0() {
        vm_assert_eq("
            PUSH13 0xc3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000000000000000000000000000000000c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push14_0() {
        vm_assert_eq("
            PUSH14 0xc2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000000000000000000000000000c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push15_0() {
        vm_assert_eq("
            PUSH15 0xc1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000000000000000000000000000c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push16_0() {
        vm_assert_eq("
            PUSH16 0xc0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000000000000000000000000000c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push17_0() {
        vm_assert_eq("
            PUSH17 0xb7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000000000000000000000b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push18_0() {
        vm_assert_eq("
            PUSH18 0xb6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000000000000000000000b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push19_0() {
        vm_assert_eq("
            PUSH19 0xb5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000000000000000000000b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push20_0() {
        vm_assert_eq("
            PUSH20 0xb4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000000000000000b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push21_0() {
        vm_assert_eq("
            PUSH21 0xb3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000000000000000b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push22_0() {
        vm_assert_eq("
            PUSH22 0xb2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000000000000000b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push23_0() {
        vm_assert_eq("
            PUSH23 0xb1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000000000b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push24_0() {
        vm_assert_eq("
            PUSH24 0xb0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000000000b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push25_0() {
        vm_assert_eq("
            PUSH25 0xa7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000000000a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push26_0() {
        vm_assert_eq("
            PUSH26 0xa6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000000000a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push27_0() {
        vm_assert_eq("
            PUSH27 0xa5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000000000a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push28_0() {
        vm_assert_eq("
            PUSH28 0xa4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00000000a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push29_0() {
        vm_assert_eq("
            PUSH29 0xa3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "000000a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push30_0() {
        vm_assert_eq("
            PUSH30 0xa2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "0000a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push31_0() {
        vm_assert_eq("
            PUSH31 0xa1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "00a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_push32_0() {
        vm_assert_eq("
            PUSH32 0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7
            retword
            ",
            "a0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7",
            None,
        );
    }

    #[test]
    fn opcode_return_0() {
        vm_assert_eq("
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
        );
    }

    #[test]
    fn opcode_return_1() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            MSTORE
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "0000000000000000000000000000000000000000000000000000000000000001",
            None,
        );
    }

    #[test]
    fn opcode_return_2() {
        vm_assert_eq("
            PUSH1 0x01
            PUSH1 0x00
            RETURN
            ",
            "00",
            None,
        );
    }

    #[test]
    fn opcode_return_3() {
        vm_assert_eq("
            0
            MLOAD
            GAS
            PUSH1 0x00
            MSTORE
            PUSH1 0x20
            PUSH1 0x00
            RETURN
            ",
            "000000000000000000000000000000000000000000000000000012309ce53ff5",
            None,
        );
    }
}
