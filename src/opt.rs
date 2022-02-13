// Copyright 2022 The Psyche Authors
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

use crate::instructions::EvmOpcode;
use crate::pex::Pex;
use crate::schedule::Schedule;
use crate::u256;
use crate::u256::U256;
use crate::vm::OPCODE_INFOS;
use crate::vm::VmRom;
use crate::utils::encode_hex;

use std::collections::{HashMap};
use std::convert::From;
use std::fmt;

type LiveRange = (isize, isize, u16, bool, i16);

#[derive(Debug, Copy, Clone)]
pub struct BlockInfo {
    pub gas: u64,
    pub stack_min_size: u16,
    pub stack_rel_max_size: u16,
    pub start_addr: (u16, u16),
    pub fall_addr: u16,
}

impl BlockInfo {
    pub fn default() -> BlockInfo {
        BlockInfo {
            gas: 0,
            stack_min_size: 0,
            stack_rel_max_size: 0,
            start_addr: (0, 0),
            fall_addr: 0,
        }
    }

    fn new(
        stack_min_size: u16,
        stack_max_size: u16,
        gas: u64,
        start_addr: u16,
        fall_addr: u16,
    ) -> BlockInfo {
        let stack_rel_max_size = if stack_max_size > stack_min_size {
            stack_max_size - stack_min_size
        } else {
            0
        };
        BlockInfo {
            gas,
            stack_min_size,
            stack_rel_max_size,
            start_addr: (start_addr, 0),
            fall_addr,
        }
    }
}

#[derive(Debug)]
enum Operand {
    Address { offset: i16, ret: bool },
    JumpDest { addr: u32 },
    Immediate { index: u16 },
    Temporary { id: u16, ret: bool }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Opcode(pub u8);

impl Opcode {
    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl From<EvmOpcode> for Opcode {
    fn from(opcode: EvmOpcode) -> Self {
        const TO_INTERNAL: [Opcode; 256] = [
            Opcode::STOP,
            Opcode::ADD,
            Opcode::MUL,
            Opcode::SUB,
            Opcode::DIV,
            Opcode::SDIV,
            Opcode::MOD,
            Opcode::SMOD,
            Opcode::ADDMOD,
            Opcode::MULMOD,
            Opcode::EXP,
            Opcode::SIGNEXTEND,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::LT,
            Opcode::GT,
            Opcode::SLT,
            Opcode::SGT,
            Opcode::EQ,
            Opcode::ISZERO,
            Opcode::AND,
            Opcode::OR,
            Opcode::XOR,
            Opcode::NOT,
            Opcode::BYTE,
            Opcode::SHL,
            Opcode::SHR,
            Opcode::SAR,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::SHA3,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,//Opcode::ADDRESS,
            Opcode::INVALID,//Opcode::BALANCE,
            Opcode::INVALID,//Opcode::ORIGIN,
            Opcode::INVALID,//Opcode::CALLER,
            Opcode::INVALID,//Opcode::CALLVALUE,
            Opcode::INVALID,//Opcode::CALLDATALOAD,
            Opcode::INVALID,//Opcode::CALLDATASIZE,
            Opcode::INVALID,//Opcode::CALLDATACOPY,
            Opcode::INVALID,//Opcode::CODESIZE,
            Opcode::INVALID,//Opcode::CODECOPY,
            Opcode::INVALID,//Opcode::GASPRICE,
            Opcode::INVALID,//Opcode::EXTCODESIZE,
            Opcode::INVALID,//Opcode::EXTCODECOPY,
            Opcode::INVALID,//Opcode::RETURNDATASIZE,
            Opcode::INVALID,//Opcode::RETURNDATACOPY,
            Opcode::INVALID,//Opcode::EXTCODEHASH,
            Opcode::INVALID,//Opcode::BLOCKHASH,
            Opcode::INVALID,//Opcode::COINBASE,
            Opcode::INVALID,//Opcode::TIMESTAMP,
            Opcode::INVALID,//Opcode::NUMBER,
            Opcode::INVALID,//Opcode::DIFFICULTY,
            Opcode::INVALID,//Opcode::GASLIMIT,
            Opcode::INVALID,//Opcode::CHAINID,
            Opcode::INVALID,//Opcode::SELFBALANCE,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,//Opcode::POP,
            Opcode::MLOAD,
            Opcode::MSTORE,
            Opcode::MSTORE8,
            Opcode::SLOAD,
            Opcode::SSTORE,
            Opcode::JUMP,
            Opcode::JUMPI,
            Opcode::PC,
            Opcode::MSIZE,//Opcode::MSIZE,
            Opcode::GAS,//Opcode::GAS,
            Opcode::INVALID,//Opcode::JUMPDEST,
            Opcode::INVALID,//Opcode::BEGINSUB,
            Opcode::INVALID,//Opcode::RETURNSUB,
            Opcode::INVALID,//Opcode::JUMPSUB,
            Opcode::INVALID,
            Opcode::MADD,//Opcode::PUSH1,
            Opcode::SET1,//Opcode::PUSH2,
            Opcode::SET2,//Opcode::PUSH3,
            Opcode::JUMPV,//Opcode::PUSH4,
            Opcode::JUMPIV,//Opcode::PUSH5,
            Opcode::INVALID,//Opcode::PUSH6,
            Opcode::INVALID,//Opcode::PUSH7,
            Opcode::INVALID,//Opcode::PUSH8,
            Opcode::INVALID,//Opcode::PUSH9,
            Opcode::INVALID,//Opcode::PUSH10,
            Opcode::INVALID,//Opcode::PUSH11,
            Opcode::INVALID,//Opcode::PUSH12,
            Opcode::INVALID,//Opcode::PUSH13,
            Opcode::INVALID,//Opcode::PUSH14,
            Opcode::INVALID,//Opcode::PUSH15,
            Opcode::INVALID,//Opcode::PUSH16,
            Opcode::INVALID,//Opcode::PUSH17,
            Opcode::INVALID,//Opcode::PUSH18,
            Opcode::INVALID,//Opcode::PUSH19,
            Opcode::INVALID,//Opcode::PUSH20,
            Opcode::INVALID,//Opcode::PUSH21,
            Opcode::INVALID,//Opcode::PUSH22,
            Opcode::INVALID,//Opcode::PUSH23,
            Opcode::INVALID,//Opcode::PUSH24,
            Opcode::INVALID,//Opcode::PUSH25,
            Opcode::INVALID,//Opcode::PUSH26,
            Opcode::INVALID,//Opcode::PUSH27,
            Opcode::INVALID,//Opcode::PUSH28,
            Opcode::INVALID,//Opcode::PUSH29,
            Opcode::INVALID,//Opcode::PUSH30,
            Opcode::INVALID,//Opcode::PUSH31,
            Opcode::INVALID,//Opcode::PUSH32,
            Opcode::INVALID,//Opcode::DUP1,
            Opcode::INVALID,//Opcode::DUP2,
            Opcode::INVALID,//Opcode::DUP3,
            Opcode::INVALID,//Opcode::DUP4,
            Opcode::INVALID,//Opcode::DUP5,
            Opcode::INVALID,//Opcode::DUP6,
            Opcode::INVALID,//Opcode::DUP7,
            Opcode::INVALID,//Opcode::DUP8,
            Opcode::INVALID,//Opcode::DUP9,
            Opcode::INVALID,//Opcode::DUP10,
            Opcode::INVALID,//Opcode::DUP11,
            Opcode::INVALID,//Opcode::DUP12,
            Opcode::INVALID,//Opcode::DUP13,
            Opcode::INVALID,//Opcode::DUP14,
            Opcode::INVALID,//Opcode::DUP15,
            Opcode::INVALID,//Opcode::DUP16,
            Opcode::INVALID,//Opcode::SWAP1,
            Opcode::INVALID,//Opcode::SWAP2,
            Opcode::INVALID,//Opcode::SWAP3,
            Opcode::INVALID,//Opcode::SWAP4,
            Opcode::INVALID,//Opcode::SWAP5,
            Opcode::INVALID,//Opcode::SWAP6,
            Opcode::INVALID,//Opcode::SWAP7,
            Opcode::INVALID,//Opcode::SWAP8,
            Opcode::INVALID,//Opcode::SWAP9,
            Opcode::INVALID,//Opcode::SWAP10,
            Opcode::INVALID,//Opcode::SWAP11,
            Opcode::INVALID,//Opcode::SWAP12,
            Opcode::INVALID,//Opcode::SWAP13,
            Opcode::INVALID,//Opcode::SWAP14,
            Opcode::INVALID,//Opcode::SWAP15,
            Opcode::INVALID,//Opcode::SWAP16,
            Opcode::INVALID,//Opcode::LOG0,
            Opcode::INVALID,//Opcode::LOG1,
            Opcode::INVALID,//Opcode::LOG2,
            Opcode::INVALID,//Opcode::LOG3,
            Opcode::INVALID,//Opcode::LOG4,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,//Opcode::CREATE,
            Opcode::INVALID,//Opcode::CALL,
            Opcode::INVALID,//Opcode::CALLCODE,
            Opcode::RETURN,
            Opcode::INVALID,//Opcode::DELEGATECALL,
            Opcode::INVALID,//Opcode::CREATE2,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,//Opcode::STATICCALL,
            Opcode::INVALID,
            Opcode::INVALID,
            Opcode::INVALID,//Opcode::REVERT,
            Opcode::INVALID,
            Opcode::INVALID,//Opcode::SELFDESTRUCT,
        ];
        TO_INTERNAL[opcode as usize]
    }
}

impl Opcode {
    pub const STOP: Opcode = Opcode(0x00);
    pub const ADD: Opcode = Opcode(0x01);
    pub const MUL: Opcode = Opcode(0x02);
    pub const SUB: Opcode = Opcode(0x03);
    pub const DIV: Opcode = Opcode(0x04);
    pub const SDIV: Opcode = Opcode(0x05);
    pub const MOD: Opcode = Opcode(0x06);
    pub const SMOD: Opcode = Opcode(0x07);
    pub const ADDMOD: Opcode = Opcode(0x08);
    pub const MULMOD: Opcode = Opcode(0x09);
    pub const EXP: Opcode = Opcode(0x0a);
    pub const SIGNEXTEND: Opcode = Opcode(0x0b);

    pub const LT: Opcode = Opcode(0x10);
    pub const GT: Opcode = Opcode(0x11);
    pub const SLT: Opcode = Opcode(0x12);
    pub const SGT: Opcode = Opcode(0x13);
    pub const EQ: Opcode = Opcode(0x14);
    pub const ISZERO: Opcode = Opcode(0x15);
    pub const AND: Opcode = Opcode(0x16);
    pub const OR: Opcode = Opcode(0x17);
    pub const XOR: Opcode = Opcode(0x18);
    pub const NOT: Opcode = Opcode(0x19);
    pub const BYTE: Opcode = Opcode(0x1a);
    pub const SHL: Opcode = Opcode(0x1b);
    pub const SHR: Opcode = Opcode(0x1c);
    pub const SAR: Opcode = Opcode(0x1d);

    pub const SHA3: Opcode = Opcode(0x20);

    pub const MLOAD: Opcode = Opcode(0x51);
    pub const MSTORE: Opcode = Opcode(0x52);
    pub const MSTORE8: Opcode = Opcode(0x53);
    pub const SLOAD: Opcode = Opcode(0x54);
    pub const SSTORE: Opcode = Opcode(0x55);
    pub const JUMP: Opcode = Opcode(0x56);
    pub const JUMPI: Opcode = Opcode(0x57);
    pub const PC: Opcode = Opcode(0x58);
    pub const MSIZE: Opcode = Opcode(0x59);
    pub const GAS: Opcode = Opcode(0x5a);

    pub const MADD: Opcode = Opcode(0x60); // reuse PUSH1
    pub const SET1: Opcode = Opcode(0x61); // reuse PUSH2
    pub const SET2: Opcode = Opcode(0x62); // reuse PUSH3
    pub const JUMPV: Opcode = Opcode(0x63); // reuse PUSH4
    pub const JUMPIV: Opcode = Opcode(0x64); // reuse PUSH5

    pub const RETURN: Opcode = Opcode(0xf3);

    pub const INVALID: Opcode = Opcode(0xfe);
}

impl Opcode {
    pub const fn mnemonic(&self) -> &'static str {
        match *self {
            Opcode::STOP => "stop",
            Opcode::ADD => "add",
            Opcode::MUL => "mul",
            Opcode::SUB => "sub",
            Opcode::DIV => "div",
            Opcode::SDIV => "sdiv",
            Opcode::MOD => "mod",
            Opcode::SMOD => "smod",
            Opcode::ADDMOD => "addmod",
            Opcode::MULMOD => "mulmod",
            Opcode::EXP => "exp",
            Opcode::SIGNEXTEND => "signextend",

            Opcode::LT => "lt",
            Opcode::GT => "gt",
            Opcode::SLT => "slt",
            Opcode::SGT => "sgt",
            Opcode::EQ => "eq",
            Opcode::ISZERO => "iszero",
            Opcode::AND => "and",
            Opcode::OR => "or",
            Opcode::XOR => "xor",
            Opcode::NOT => "not",
            Opcode::BYTE => "byte",
            Opcode::SHL => "shl",
            Opcode::SHR => "shr",
            Opcode::SAR => "sar",

            Opcode::SHA3 => "sha3",

            Opcode::MLOAD => "mload",
            Opcode::MSTORE => "mstore",
            Opcode::MSTORE8 => "mstore8",
            Opcode::SLOAD => "sload",
            Opcode::SSTORE => "sstore",
            Opcode::JUMP => "jump",
            Opcode::JUMPI => "jumpi",
            Opcode::PC => "pc",
            Opcode::MSIZE => "msize",
            Opcode::GAS => "gas",

            Opcode::MADD => "madd",
            Opcode::SET1 => "set1",
            Opcode::SET2 => "set2",
            Opcode::JUMPV => "jumpv",
            Opcode::JUMPIV => "jumpiv",

            Opcode::RETURN => "return",

            Opcode::INVALID => "invalid",

            _ => unimplemented!()
        }
    }

    /// Remaining bits in the instruction to store sp_offset
    pub const fn sp_offset_bits(&self) -> usize {
        match *self {
            // 8 11 15 15 15
            Opcode::ADDMOD | Opcode::MULMOD | Opcode::MADD => 0,
            // 8 11 11 15 15
            Opcode::SET2 => 4,
            _ => 11,
        }
    }
}

#[derive(Debug)]
pub struct Instr {
    opcode: Opcode,
    operands: Vec<Operand>,
    sp_offset: i16,
}

struct InstrWithConsts<'a> {
    instr: &'a Instr,
    consts: &'a [U256],
}

struct InstrWithConstsAndBlockInfos<'a> {
    instr: &'a Instr,
    consts: &'a [U256],
    blocks: &'a [BlockInfo],
}

impl Instr {
    fn with_imms<'a>(instr: &'a Instr, consts: &'a [U256]) -> InstrWithConsts<'a> {
        InstrWithConsts {
            instr,
            consts,
        }
    }

    fn with_imms_and_block_infos<'a>(instr: &'a Instr, consts: &'a [U256], blocks: &'a[BlockInfo]) -> InstrWithConstsAndBlockInfos<'a> {
        InstrWithConstsAndBlockInfos {
            instr,
            consts,
            blocks,
        }
    }

    fn invalid() -> Instr {
       Instr {
            opcode: Opcode::INVALID,
            operands: vec!(),
            sp_offset: 0,
        }
    }

    fn new(
        opcode: EvmOpcode,
        valid_jumpdests: *const u64,
        retarg: Option<Argument>,
        args: &[Argument],
        imms: &mut Vec<U256>,
    ) -> Instr {
        let is_jumpdest = |addr: u64| {
            let addr = (addr as isize) % (Pex::BYTECODE_SIZE as isize);
            let bits = unsafe { *valid_jumpdests.offset(addr / 64) };
            let mask = 1 << (addr % 64);
            (bits & mask) > 0
        };
        match opcode {
            EvmOpcode::JUMP => {
                if let Argument::Immediate { value } = args[0] {
                    let in_bounds = unsafe {
                        u256::is_ltpow2_u256(value, Pex::BYTECODE_SIZE)
                    };
                    let low = value.low_u64();
                    let opcode = if in_bounds & is_jumpdest(low) {
                        // Target is statically known and valid
                        Opcode::JUMPV
                    } else {
                        Opcode::JUMP
                    };
                    return Instr {
                        opcode,
                        operands: vec![Operand::JumpDest { addr: low as u32 }],
                        sp_offset: 0,
                    }
                }
            },
            EvmOpcode::JUMPI => {
                if let Argument::Immediate { value } = args[0] {
                    let in_bounds = unsafe {
                        u256::is_ltpow2_u256(value, Pex::BYTECODE_SIZE)
                    };
                    let low = value.low_u64();
                    let opcode = if in_bounds & is_jumpdest(low) {
                        // Target is statically known and valid
                        Opcode::JUMPIV
                    } else {
                        Opcode::JUMPI
                    };
                    return Instr {
                        opcode,
                        operands: vec![
                            Operand::JumpDest { addr: low as u32 },
                            args[1].to_operand(imms, false)
                        ],
                        sp_offset: 0,
                    }
                }
            },
            _ => (),
        }
        //
        let mut v = vec![];
        if let Some(arg) = retarg {
            v.push(arg.to_operand(imms, true));
        }
        for a in args {
            v.push((*a).to_operand(imms, false));
        }
        Instr {
            opcode: Opcode::from(opcode),
            operands: v,
            sp_offset: 0,
        }
    }

    fn set1(dst: Argument, src: Argument, imms: &mut Vec<U256>) -> Instr {
        let mut v = vec![];
        v.push(dst.to_operand(imms, true));
        v.push(src.to_operand(imms, false));
        Instr {
            opcode: Opcode::SET1,
            operands: v,
            sp_offset: 0,
        }
    }

    fn set2(dst0: Argument, dst1: Argument, src0: Argument, src1: Argument, imms: &mut Vec<U256>) -> Instr {
        let mut v = vec![];
        v.push(dst0.to_operand(imms, true));
        v.push(dst1.to_operand(imms, true));
        v.push(src0.to_operand(imms, false));
        v.push(src1.to_operand(imms, false));
        Instr {
            opcode: Opcode::SET2,
            operands: v,
            sp_offset: 0,
        }
    }

    /// len is in multiple of 64-bit words
    fn len(&self) -> usize {
        if self.operands.len() <= 4 {
            1
        } else {
            2
        }
    }

    pub fn encode(&self, dst: *mut u64) -> usize {
        let len = self.len();
        if len == 1 {
            let mut bits: u64 = 0;
            let mut i = 0;
            bits |= self.opcode.to_u8() as u64;
            i += 8;
            let offset_bits = self.opcode.sp_offset_bits();
            let base: u64 = 2;
            let to_add = base.pow(offset_bits as u32 - 1) as i16;
            bits |= ((self.sp_offset + to_add) as u64) << i;
            // TODO: assert on sp_offset
            i += offset_bits;
            for opr in &self.operands {
                match *opr {
                    Operand::Address { offset, ret } => {
                        assert!(offset >= -1024 && offset < 1024);
                        let offset = (offset + 1024) as u16;
                        let offset = offset & 0x7ff; // clear upper bits
                        bits |= (offset as u64) << i;
                        if ret {
                            // it's an offset to a destination address, stored
                            // on a u16 but the value range is [-1024, +1023]
                            // and so can fit 11 bits
                            i += 11;
                        } else {
                            // this need to be 15 bits, because we can store
                            // immediate as well (1-bit + 14-bit index)
                            i += 15;
                        }
                    },
                    Operand::Immediate { index } => {
                        // check that the index addresses 16K entries
                        assert!(index < 0x4000);
                        // set bit 14 to indicate for an immediate value
                        let index = (index + 1024) | 0x4000;
                        bits |= (index as u64) << i;
                        i += 15
                    },
                    Operand::JumpDest { addr } => {
                        // TODO: fix this
                        // check that the address fits on 19-bits
                        //assert!(addr < 0x80000);
                        bits |= (addr as u64) << i;
                        i += 16
                    },
                    Operand::Temporary { id: _, ret: _ } => {
                        unreachable!()
                    },
                }
            }
            unsafe {
                *dst = bits;
            }
        } else {
            unimplemented!()
        }
        len
    }
}

impl<'a> fmt::Display for InstrWithConsts<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.instr.opcode.mnemonic();
        let res = write!(f, "{:<8} ", s);

        for opr in self.instr.operands.iter() {
            match opr {
                Operand::Immediate { index } => {
                    let value = self.consts[*index as usize];
                    write!(f, "${}, ", value.0[0]);
                },
                Operand::Address { offset, ret: _ } => {
                    write!(f, "@{:+}, ", offset);
                },
                Operand::JumpDest { addr } => {
                    write!(f, "{:02x}h, ", addr);
                },
                Operand::Temporary { id, ret:_ } => {
                    write!(f, "r{}, ", id);
                },
            }
        }
        let sp_offset = self.instr.sp_offset;
        if sp_offset != 0 {
            write!(f, "({:+})", sp_offset);
        }
        res
    }
}

impl<'a> fmt::Display for InstrWithConstsAndBlockInfos<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.instr.opcode.mnemonic();
        let res = write!(f, "{:<8} ", s);

        for (i, opr) in self.instr.operands.iter().enumerate() {
            match opr {
                Operand::Immediate { index } => {
                    let value = self.consts[*index as usize];
                    write!(f, "${}", value.0[0]);
                },
                Operand::Address { offset, ret: _ } => {
                    write!(f, "@{:+}", offset);
                },
                Operand::JumpDest { addr } => {
                    let mut instr_addr = 0;
                    // TODO: use binary search
                    for b in self.blocks {
                        if *addr as u16 == b.start_addr.0 {
                            instr_addr = b.start_addr.1 * 8;
                            break;
                        }
                    }
                    write!(f, "{:02x}h", instr_addr);
                },
                Operand::Temporary { id, ret:_ } => {
                    write!(f, "r{}", id);
                },
            }
            if i != self.instr.operands.len()-1 {
                write!(f, ", ");
            }
        }
        let sp_offset = self.instr.sp_offset;
        if sp_offset != 0 {
            write!(f, " ({:+})", sp_offset);
        }
        res
    }
}


#[derive(Debug, Copy, Clone)]
enum Argument {
    Immediate { value: U256 },
    Input { id: u16, address: i16 },
    Temporary { id: u16 },
}

struct StaticStack {
    size: usize,
    args: Vec<Argument>,
    rcs: HashMap<u16, usize>,
    lifetimes: HashMap<u16, (isize, Option<isize>)>,
    next_id: u16,
}

impl Argument {
    fn to_operand(&self, imms: &mut Vec<U256>, ret: bool) -> Operand {
        match *self {
            Argument::Immediate { value } => {
                // TODO: assert if u16 is too small (should not happen)
                let index = imms.len() as u16;
                imms.push(value);
                Operand::Immediate { index }
            },
            Argument::Input { id: _, address } => {
                Operand::Address { offset: address, ret }
            },
            Argument::Temporary { id } => {
                Operand::Temporary { id, ret }
            },
        }
    }
}

impl StaticStack {
    fn new() -> StaticStack {
        const CAPACITY: usize = 1024;
        StaticStack {
            size: 0,
            args: Vec::with_capacity(CAPACITY),
            rcs: HashMap::with_capacity(CAPACITY),
            lifetimes: HashMap::with_capacity(CAPACITY),
            next_id: 0,
        }
    }

    fn len(&self) -> usize {
        self.args.len()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn diff(&self) -> isize {
        self.len() as isize - self.size() as isize
    }

    fn clear(&mut self, size: usize) {
        self.size = size;
        self.args.clear();
        self.rcs.clear();
        self.lifetimes.clear();
        self.next_id = 0;

        for i in 0..size {
            let address = (i as isize - size as isize) as i16;

            self.push(Argument::Input { id: self.next_id, address }, -1);
            //println!("push i{} @{}", self.next_id, address);
            self.next_id += 1;
        }
    }

    fn push(&mut self, arg: Argument, pc: isize) -> &mut Self {
        let id = match arg {
            Argument::Input { id, address: _ } => Some(id),
            Argument::Temporary { id } => Some(id),
            Argument::Immediate { value: _ } => None,
        };
        if let Some(id) = id {
            if let Some(v) = self.rcs.get_mut(&id) {
                // increment refcount
                *v = *v + 1;
            } else {
                // it's a new argument, insert with refcount that is set to 1
                self.rcs.insert(id, 1);
                //println!("inserting {} at {}", id, pc);
                self.lifetimes.insert(id, (pc as isize, None));
            }
        }
        //println!("pushing on the stack {:?}", arg);
        self.args.push(arg);
        self
    }

    fn pop(&mut self, pc: isize) -> (&mut Self, Argument) {
        //println!("len: {}", self.args.len());
        // TODO: explain why it's safe to unwrap
        let arg = self.args.pop().unwrap();
        let id = match arg {
            Argument::Input { id, address: _ } => Some(id),
            Argument::Temporary { id } => Some(id),
            Argument::Immediate { value: _ } => None,
        };
        if let Some(id) = id {
            let v = self.rcs.get_mut(&id).unwrap();
            // decrement the refcount
            let rc = *v - 1;
            //println!("popping {} (rc={})", id, rc);
            if rc == 0 {
                // the register is not in use anymore, remove rc and mark
                // lifetime end
                self.rcs.remove(&id);
                let (start, end) = self.lifetimes.get_mut(&id).unwrap();
                *end = if *start != pc {
                    Some(pc as isize)
                } else {
                    Some(pc as isize + 1)
                };
            } else {
                *v = rc;
            };
        }
        (self, arg)
    }

    fn swap(&mut self, index: usize) -> &mut Self {
        let n = self.args.len() - 1 - 1 - index;
        //println!("n: {}", n);
        // TODO: explain why it's safe to unwrap
        let temp = self.args.get(n).unwrap();
        let temp = temp.clone();
        let top = self.args.len() - 1;
        //println!("top: {}", top);
        self.args[n] = self.args[top];
        self.args[top] = temp;
        self
    }

    fn dup(&mut self, index: usize) -> &mut Self {
        let n = self.args.len() - 1 - index;
        // TODO: explain why it's safe to unwrap
        let arg = self.args.get(n).unwrap();
        let id = match arg {
            Argument::Input { id, address: _ } => Some(id),
            Argument::Temporary { id } => Some(id),
            Argument::Immediate { value: _ } => None,
        };
        if let Some(id) = id {
            let v = self.rcs.get_mut(id).unwrap();
            *v = *v + 1;
        }
        let arg = arg.clone();
        self.args.push(arg);
        self
    }

    // Allocate a new temporary.
    fn alloc_temporary(&mut self) -> (&mut Self, Argument) {
        let arg = Argument::Temporary { id: self.next_id };
        self.next_id += 1;
        (self, arg)
    }

    fn eval_opcode(
        &mut self,
        opcode: EvmOpcode,
        valid_jumpdests: *const u64,
        pc: isize,
        imms: &mut Vec<U256>,
    ) -> Result<Instr, &str> {
        let (delta, alpha) = opcode.delta_alpha();
        assert!(alpha == 0 || alpha == 1);
        // pop delta arguments off the stack
        let mut args = [ Argument::Immediate { value: U256::default() }; 7];
        let stack = (0..delta).fold(Ok(self), |res, i| {
            if let Ok(stack) = res {
                let (stack, arg) = stack.pop(pc);
                args[i] = arg;
                Ok(stack)
            } else {
                res
            }
        })?;
        // alloc temporary and push it to the stack if alpha == 1
        let (stack, reg) = if alpha > 0 {
            let (stack, reg) = stack.alloc_temporary();
            let stack = stack.push(reg, pc as isize);
            (stack, Some(reg))
        } else {
            (stack, None)
        };
        Ok(Instr::new(opcode, valid_jumpdests, reg, &args[0..delta], imms))
    }

    fn eval_block<'a>(
        &mut self,
        bytecode: &[u8],
        valid_jumpdests: *const u64,
        imms: &mut Vec<U256>,
        instrs: &mut Vec<Instr>,
    ) {
        let mut block_pc = -1;
        let mut i = 0;
        while i < bytecode.len() {
            let opcode = unsafe { std::mem::transmute::<_, EvmOpcode>(bytecode[i]) };
            // handle stack opcodes first
            // TODO: use a match expr
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                let start = i + 1;
                let end = start + num_bytes;
                let mut buffer: [u8; 32] = [0; 32];
                VmRom::swap_bytes(&bytecode[start..end], &mut buffer);
                let value = U256::from_slice(unsafe { std::mem::transmute::<_, &[u64; 4]>(&buffer) });
                self.push(Argument::Immediate { value }, block_pc);
                i += num_bytes;
            } else if opcode.is_dup() {
                let index = opcode.dup_index();
                self.dup(index);
            } else if opcode.is_swap() {
                let index = opcode.swap_index();
                self.swap(index);
            } else if opcode == EvmOpcode::POP {
                self.pop(block_pc);
            } else if opcode == EvmOpcode::JUMPDEST {
                // do nothing
                ()
            } else {
                // handle non-stack opcodes
                block_pc += 1;
                let res = self.eval_opcode(opcode, valid_jumpdests, block_pc, imms);
                if let Ok(instr) = res {
                    instrs.push(instr);
                } else {
                    instrs.push(Instr::invalid());
                }
            }
            i += 1;
        }
    }

    fn print_lifetimes(stack: &StaticStack, instr_len: usize) {
        let end_pc = instr_len as isize -1;
        println!("lifetimes: {}", instr_len);
        let mut sorted_lifetimes: Vec<(isize, isize, u16, bool)> = vec!();
        for (k, v) in &stack.lifetimes {
            let id = k;

            let (start, end) = v;
            let end = end.unwrap_or(end_pc);
            let is_input = (*id as usize) < stack.size();
            if is_input {
                println!("i{}: {} to {:?}", id, start, end);
            } else {
                println!("r{}: {} to {:?}", id, start, end);
            }
            sorted_lifetimes.push((*start, end, *id, is_input));
        }
        // sorted by end of life
        sorted_lifetimes.sort_by_key(|v| v.1);
        println!("sorted: {:?}", sorted_lifetimes);
    }

    fn alloc_stack_slots(
        &mut self,
        imms: &[U256],
        instrs: &mut [Instr],
        instr_len: usize,
        block_info: &BlockInfo,
    ) {
        // for arg in stack.args.iter() {
        //     println!(">> {:?}", arg);
        // }
        let print_log = false;
        if print_log {
            for (i, instr) in instrs.iter().enumerate() {
                println!("{:02}: {}", i, Instr::with_imms(instr, &imms));
            }
        }

        let diff = self.len() as isize - self.size() as isize;
        //println!("diff: {}", diff);

        let mut constraints: HashMap<u16, i16> = HashMap::new();

        let mut ref_address = diff - 1;
        for arg in self.args.iter().rev() {
            //println!("{:?}", arg);
            match arg {
                Argument::Temporary { id } => {
                    //println!("need to allocate @{} to temporary r{}", ref_address, id);
                    constraints.insert(*id, ref_address as i16);
                },
                _ => (),
            }
            ref_address -= 1;
        }
        if self.args.is_empty() {
            //println!("nothing to do because stack at the end is empty");
        }

        //Self::print_lifetimes(self, instr_len);

        let end_pc = instr_len as isize;
        //println!("lifetimes:");
        let mut sorted_lifetimes: Vec<(isize, isize, u16, bool, Option<i16>)> = vec!();
        for (k, v) in &self.lifetimes {
            let id = k;
            let (start, end) = v;
            let end = end.unwrap_or(end_pc);
            let is_input = (*id as usize) < self.size();
            let addr = if is_input {
                let size = block_info.stack_min_size as i16;
                //println!("size {}", size);
                let addr = (*id as isize - size as isize) as i16;
                Some(addr)
            } else {
                None
            };
            sorted_lifetimes.push((*start, end, *id, is_input, addr));
        }
        // sort by end of life, in case of a tie use start of life
        {
            use std::cmp::Ordering;
            sorted_lifetimes.sort_by(|a, b| {
                match a.1.cmp(&b.1) {
                    Ordering::Equal => a.0.cmp(&b.0),
                    other => other,
                }
            });
        }
        //println!("sorted: {:?}", sorted_lifetimes);

        let mut free_slots: Vec<i16> = vec!();
        for i in 0..block_info.stack_rel_max_size {
            free_slots.push(i as i16);
        }
        let mut pc: isize = 0;
        let mut start_idx = 0;
        while pc < instr_len as isize {
            if print_log { println!("pc: {}", pc) };
            if print_log { println!("free slots: {:?}", free_slots) };
            if print_log { println!("sorted: {:?}", sorted_lifetimes) };
            //let mut max = 0;
            for v in &mut sorted_lifetimes[start_idx..] {
                let (start, end, id, is_input, addr) = *v;
                if pc == end {
                    let addr = addr.expect(&format!("unallocated register {}{}", if is_input { "i" } else { "r" }, id));
                    if print_log { println!("{}{} has reached end of life, its address @{} is available for writing",
                        if is_input { "i" } else { "r" }, id, addr) };
                    assert!(!free_slots.contains(&addr), "@{} is present in free slots", addr);
                    free_slots.push(addr);
                }
                if pc == start {
                    if print_log { println!("{}{} is now alive and need to be allocated to a stack slot!",
                        if is_input { "i" } else { "r" }, id) };
                    if !is_input {
                        let addr = if let Some(addr) = constraints.get(&id) {
                            if print_log { println!("constraining it to @{}", addr) };
                            let idx = free_slots.iter().position(|&x| x == *addr).unwrap();
                            free_slots.remove(idx);
                            *addr
                        } else {
                            // no particular constraint, pick what's free
                            let addr = free_slots.pop().unwrap();
                            if print_log { println!("found free @{}", addr) };
                            addr
                        };
                        v.4 = Some(addr);
                    }
                }
            }
            pc += 1;
        }
        //println!("{:?}", bb);

        // patch instruction temporaries with their allocated stack slots
        for instr in instrs {
            for opr in &mut instr.operands {
                match *opr {
                    Operand::Temporary { id, ret } => {
                        let res = sorted_lifetimes.iter().find(|&tu| tu.2 == id);
                        let (_,_,_,_,addr) = res.unwrap();
                        *opr = Operand::Address {
                            offset: (*addr).unwrap(),
                            ret,
                        };
                    },
                    _ => (),
                }
            }
        }
    }

    fn block_fixup(&mut self, imms: &mut Vec<U256>, instrs: &mut Vec<Instr>) {
        let diff = self.len() as isize - self.size() as isize;
        let mut sets = self.args
            .iter()
            .rev()
            .enumerate()
            .filter_map(|(i, arg)| {
                let ref_address = diff - 1 - i as isize;
                match arg {
                    Argument::Immediate { value } => {
                        Some((Argument::Input { id: u16::MAX, address: ref_address as i16 },
                            Argument::Immediate { value: *value }
                        ))
                    },
                    Argument::Input { id, address } => {
                        if ref_address == *address as isize {
                            // input was unmodified, do nothing
                            None
                        } else  {
                            Some((Argument::Input { id: u16::MAX, address: ref_address as i16 },
                                Argument::Input { id: *id, address: *address }
                            ))
                        }
                    },
                    _ => None,
                }
            });

        // We need to determine if the last instruction breaks the control
        // flow, if yes then we can't just push back the regularization
        // instruction(s) (to patch the stack and/or stack pointer)
        let save = |instrs: &mut Vec<Instr>| {
            if let Some(instr) = instrs.last() {
                match instr.opcode {
                    Opcode::JUMP | Opcode::JUMPI | Opcode::JUMPV | Opcode::JUMPIV
                    | Opcode::RETURN => instrs.pop(),
                    _ => None,
                }
            } else {
                None
            }
        };
        let restore = |instrs: &mut Vec<Instr>, to_push: Option<Instr>| {
            if let Some(instr) = to_push {
                instrs.push(instr);
            }
        };

        let x = save(instrs);

        loop {
            let s0 = sets.next();
            let s1 = sets.next();
            match (s0, s1) {
                (Some((dst0, src0)), Some((dst1, src1))) => {
                    instrs.push(Instr::set2(dst0, dst1, src0, src1, imms));
                },
                (Some((dst, src)), None) => {
                    instrs.push(Instr::set1(dst, src, imms));
                    break;
                },
                (None, None) => break,
                (None, Some(_)) => unreachable!(),
            }
        }

        // Restore the control flow instruction
        restore(instrs, x);

        if diff != 0 {
            // We need to store in the last instruction of the block the stack
            // pointer offset
            let push_set1 = if let Some(instr) = instrs.last_mut() {
                instr.opcode.sp_offset_bits() < 11
            } else {
                true
            };
            if push_set1 {
                // We can't handle this situation right now and therefore must
                // assert, iow we should always be able to fit sp_offset in
                // those instructions
                let ctrl_flow = if let Some(instr) = instrs.last() {
                    match instr.opcode {
                        Opcode::JUMP | Opcode::JUMPI | Opcode::JUMPV | Opcode::JUMPIV
                        | Opcode::RETURN => true,
                        _ => false,
                    }
                } else {
                    false
                };
                assert!(!ctrl_flow);

                instrs.push(Instr::set1(
                    Argument::Input { id: 0, address: 0},
                    Argument::Input { id: 0, address: 0},
                    imms,
                ));
            }
            let instr: &mut Instr = instrs.last_mut().unwrap();
            instr.sp_offset = diff as i16;
        }
    }
}

pub fn build_valid_jumpdests(
    bytecode: &[u8],
    valid_jump_dests: *mut u64
) {
    let mut i = 0;
    let mut j = 0;
    let mut bits: u64 = 0;
    while i < bytecode.len() {
        let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(bytecode[i]) };
        if opcode == EvmOpcode::JUMPDEST {
            bits |= 1 << (i % 64);
        }
        if opcode.is_push() {
            let num_bytes = opcode.push_index() + 1;
            i += num_bytes;
        }
        i += 1;

        if ((i / 64) > j) | (i >= bytecode.len()) {
            unsafe { *valid_jump_dests.offset(j as isize) = bits; }
            bits = 0;
            j += 1;
        }
    }
}

pub fn build_block_infos(
    bytecode: &[u8],
    schedule: &Schedule,
    block_infos: &mut Vec<BlockInfo>
) {
    use std::cmp::max;
    #[derive(Debug)]
    struct ForwardBlock {
        addr: u32,
        stack_min_size: u16,
        stack_max_size: u16,
        stack_end_size: u16,
        gas: u64,
        is_basic_block: bool,
    }
    impl ForwardBlock {
        fn basic(
            addr: u32,
            stack_min_size: u16,
            stack_max_size: u16,
            stack_end_size: u16,
            gas: u64,
        ) -> ForwardBlock {
            ForwardBlock {
                addr,
                stack_min_size,
                stack_max_size,
                stack_end_size,
                gas,
                is_basic_block: true,
            }
        }
        fn partial(
            addr: u32,
            stack_min_size: u16,
            stack_max_size: u16,
            stack_end_size: u16,
            gas: u64,
        ) -> ForwardBlock {
            ForwardBlock {
                addr,
                stack_min_size,
                stack_max_size,
                stack_end_size,
                gas,
                is_basic_block: false,
            }
        }
    }
    let mut addr: u32 = 0;
    let mut stack_size: u16 = 0;
    let mut stack_min_size: u16 = 0;
    let mut stack_max_size: u16 = 0;
    let mut gas: u64 = 0;
    let mut fwd_blocks: Vec<ForwardBlock> = Vec::with_capacity(1024);

    // forward pass over the bytecode
    let mut i: usize = 0;
    while i < bytecode.len() {
        let code = bytecode[i];
        let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(code) };
        let (_, fee, delta, alpha) = OPCODE_INFOS[code as usize];
        // new_stack_size is (stack_size + needed + alpha) - delta
        // and represents the new stack size after the opcode has been
        // dispatched
        let (new_stack_size, needed) = if delta > stack_size {
            (alpha, (delta - stack_size))
        } else {
            // case stack_size >= delta
            ((stack_size - delta).saturating_add(alpha), 0)
        };
        stack_size = new_stack_size;
        stack_min_size = stack_min_size.saturating_add(needed);
        stack_max_size = max(stack_max_size, new_stack_size);
        // TODO: overflow possible?
        gas += fee.gas(schedule) as u64;
        if opcode.is_push() {
            let num_bytes = opcode.push_index() + 1;
            i += 1 + num_bytes;
        } else {
            i += 1;
        }
        if opcode.is_terminator() || i >= bytecode.len() {
            fwd_blocks.push(ForwardBlock::basic(
                addr,
                stack_min_size,
                stack_max_size,
                stack_size,
                gas,
            ));
            addr = i as u32;
            stack_size = 0;
            stack_min_size = 0;
            stack_max_size = 0;
            gas = 0;
        } else {
            let code = bytecode[i];
            let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(code) };
            if opcode == EvmOpcode::JUMPDEST {
                fwd_blocks.push(ForwardBlock::partial(
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_size,
                    gas,
                ));
                addr = i as u32;
                stack_size = 0;
                stack_min_size = 0;
                stack_max_size = 0;
                gas = 0;
            }
        }
    }

    // backward pass, write fwd blocks to block infos
    block_infos.resize(fwd_blocks.len(), BlockInfo::default());
    let mut i: isize = fwd_blocks.len() as isize - 1;
    let mut fall_addr = 0;
    let mut fall_addr_last = 0;
    for info in fwd_blocks.iter().rev() {
        if info.is_basic_block {
            stack_min_size = info.stack_min_size;
            stack_max_size = info.stack_max_size;
            gas = info.gas;
            fall_addr = fall_addr_last;
            fall_addr_last = info.addr as u16;
        } else {
            let (more, needed) = if stack_min_size > info.stack_end_size {
                (0, (stack_min_size - info.stack_end_size))
            } else {
                // case info.stack_end_size >= stack_min_size
                (info.stack_end_size - stack_min_size, 0)
            };
            stack_min_size = info.stack_min_size.saturating_add(needed);
            stack_max_size = max(
                info.stack_max_size.saturating_add(needed),
                stack_max_size.saturating_add(more),
            );
            gas += info.gas;
        }

        // write result
        let start_addr = info.addr as u16;
        let block_info =
            BlockInfo::new(stack_min_size, stack_max_size, gas, start_addr, fall_addr);
        block_infos[i as usize] = block_info;
        i -= 1;
    }
}

pub fn build_super_instructions(
    bytecode: &[u8],
    valid_jumpdests: *const u64,
    block_infos: &mut [BlockInfo],
    imms: &mut Vec<U256>,
    instrs: &mut Vec<Instr>,
) {
    let mut stack = StaticStack::new();
    let mut start_instr = 0;
    let mut super_block_offset = 0;

    let mut block_offset: isize = 0;
    for i in 0..block_infos.len() {
        //println!("\n==== block #{} ====", i);
        let block_info = block_infos[i];
        //println!("{:?}", block_info);

        let block_len = if i < (block_infos.len()-1) {
            let next_block_info = block_infos[i+1];
            next_block_info.start_addr.0 - block_info.start_addr.0
        } else {
            bytecode.len() as u16 - block_info.start_addr.0
        } as isize;

        // let mut offset: isize = 0;
        // while offset < block_len {
        //     let opcode = bytecode[(block_offset + offset) as usize];
        //     let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(opcode) };
        //     if opcode.is_push() {
        //         let num_bytes = opcode.push_index() as isize + 1;
        //         let start = (block_offset + offset + 1) as usize;
        //         let end =   (block_offset + offset + 1 + num_bytes) as usize;
        //         let s = encode_hex(&bytecode[start..end]);
        //         println!("{:?} 0x{}", opcode, s);
        //         offset += num_bytes;
        //     } else {
        //         println!("{:?}", opcode);
        //     }
        //     offset += 1;
        // }

        // build super instructions
        stack.clear(block_info.stack_min_size as usize);
        let block = &bytecode[block_offset as usize..(block_offset + block_len) as usize];
        stack.eval_block(block, valid_jumpdests, imms, instrs);

        let block_instr_len = instrs.len() - start_instr;
        stack.alloc_stack_slots(&imms, &mut instrs[start_instr..], block_instr_len, &block_info);
        stack.block_fixup(imms, instrs);

        // patch jump addresses and stack diff
        let block_info = &mut block_infos[i];
        block_info.start_addr.1 = super_block_offset;
        for instr in &instrs[start_instr..] {
            super_block_offset += instr.len() as u16;
        }

        // println!("\n==== block #{} ====", i);
        // println!("{:?}", block_info);

        // let mut offset: isize = 0;
        // while offset < block_len {
        //     let opcode = bytecode[(block_offset + offset) as usize];
        //     let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(opcode) };
        //     println!("{:?}", opcode);
        //     if opcode.is_push() {
        //         let num_bytes = opcode.push_index() as isize + 1;
        //         offset += num_bytes;
        //     }
        //     offset += 1;
        // }

        // println!("--");
        // for instr in &instrs[start_instr..] {
        //     let ic = Instr::with_imms(instr, &imms);
        //     println!("{}", ic);
        // }
        // println!("--");

        start_instr = instrs.len();

        block_offset += block_len;
    }

    println!("");
    let mut addr = 0;
    for instr in instrs.iter() {
        let ic = Instr::with_imms_and_block_infos(instr, &imms, &block_infos);
        println!("0x{:04x}: {}", addr, ic);
        addr += instr.len() * 8;
    }
}
