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
use crate::schedule::Schedule;
use crate::vm::VmRom;
use crate::vm::BbInfo;
use crate::u256;
use crate::u256::U256;

use std::collections::{HashMap};
use std::convert::From;
use std::fmt;

type Lifetime = (isize, isize, u16, bool, i16);

#[derive(Debug)]
enum Operand {
    Address { offset: i16 },
    JumpDest { addr: u16 },
    Immediate { index: u16 },
    Temporary { id: u16 }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Opcode(pub u8);

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
            Opcode::SET2,//Opcode::PUSH2,
            Opcode::JUMPV,//Opcode::PUSH3,
            Opcode::JUMPIV,//Opcode::PUSH4,
            Opcode::INVALID,//Opcode::PUSH5,
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
    pub const SET2: Opcode = Opcode(0x61); // reuse PUSH2
    pub const JUMPV: Opcode = Opcode(0x62); // reuse PUSH3
    pub const JUMPIV: Opcode = Opcode(0x63); // reuse PUSH4

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
            Opcode::SET2 => "set2",
            Opcode::JUMPV => "jumpv",
            Opcode::JUMPIV => "jumpiv",

            Opcode::RETURN => "return",

            Opcode::INVALID => "invalid",

            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
struct Instr {
    opcode: Opcode,
    operands: Vec<Operand>,
    sp_offset: i16,
}

struct InstrWithConsts<'a> {
    instr: &'a Instr,
    consts: &'a [U256],
}

impl Instr {
    fn with_consts<'a>(instr: &'a Instr, consts: &'a [U256]) -> InstrWithConsts<'a> {
        InstrWithConsts {
            instr,
            consts,
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
        retarg: Option<Argument>,
        args: &[Argument],
        rom: &VmRom,
        imms: &mut Vec<U256>,
    ) -> Instr {
        match opcode {
            EvmOpcode::JUMP => {
                if let Argument::Immediate { value } = args[0] {
                    let in_bounds = unsafe {
                        u256::is_ltpow2_u256(value, VmRom::MAX_CODESIZE)
                    };
                    let low = value.low_u64();
                    if in_bounds & rom.is_jumpdest(low) {
                        // Target is statically known and valid
                        return Instr {
                            opcode: Opcode::JUMPV,
                            operands: vec![Operand::JumpDest { addr: low as u16 }],
                            sp_offset: 0,
                        }
                    } else {
                        // TODO: create invalid jumpdest
                        return Instr::invalid();
                    }
                }
            },
            EvmOpcode::JUMPI => {
                if let Argument::Immediate { value } = args[0] {
                    let in_bounds = unsafe {
                        u256::is_ltpow2_u256(value, VmRom::MAX_CODESIZE)
                    };
                    let low = value.low_u64();
                    if in_bounds & rom.is_jumpdest(low) {
                        // Target is statically known and valid
                        return Instr {
                            opcode: Opcode::JUMPIV,
                            operands: vec![
                                Operand::JumpDest { addr: low as u16 },
                                args[1].to_operand(imms)
                            ],
                            sp_offset: 0,
                        }
                    } else {
                        // TODO: create invalid jumpdest
                        return Instr::invalid();
                    }
                }
            },
            _ => (),
        }
        //
        let mut v = vec![];
        if let Some(arg) = retarg {
            v.push(arg.to_operand(imms));
        }
        for a in args {
            v.push((*a).to_operand(imms));
        }
        Instr {
            opcode: Opcode::from(opcode),
            operands: v,
            sp_offset: 0,
        }
    }

    fn set2(dst0: Argument, dst1: Argument, src0: Argument, src1: Argument, imms: &mut Vec<U256>) -> Instr {
        let mut v = vec![];
        v.push(dst0.to_operand(imms));
        v.push(dst1.to_operand(imms));
        v.push(src0.to_operand(imms));
        v.push(src1.to_operand(imms));
        Instr {
            opcode: Opcode::SET2,
            operands: v,
            sp_offset: 0,
        }
    }

    fn size(&self) -> usize {
        8
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
                Operand::Address { offset } => {
                    write!(f, "@{:+}, ", offset);
                },
                Operand::JumpDest { addr } => {
                    write!(f, "{:02x}h, ", addr);
                },
                _ => panic!("only immediate, address or jumpdest are valid")
            }
        }
        let sp_offset = self.instr.sp_offset;
        if sp_offset != 0 {
            write!(f, "({:+})", sp_offset);
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
    fn to_operand(&self, imms: &mut Vec<U256>) -> Operand {
        match *self {
            Argument::Immediate { value } => {
                // TODO: assert if u16 is too small (should not happen)
                let index = imms.len() as u16;
                imms.push(value);
                Operand::Immediate { index }
            },
            Argument::Input { id: _, address } => {
                Operand::Address { offset: address }
            },
            Argument::Temporary { id } => {
                Operand::Temporary { id }
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
                let (_, end) = self.lifetimes.get_mut(&id).unwrap();
                *end = Some(pc as isize);
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
        rom: &VmRom,
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
        Ok(Instr::new(opcode, reg, &args[0..delta], rom, imms))
    }

    fn eval_block<'a>(
        &mut self,
        bytecode: &[u8],
        rom: &VmRom,
        imms: &mut Vec<U256>,
        instrs: &mut Vec<Instr>,
    ) {
        let mut block_pc = 0;
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
                let res = self.eval_opcode(opcode, &rom, block_pc, imms);
                block_pc += 1;
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
        instrs: &mut [Instr],
        instr_len: usize,
        block_info: &BbInfo,
    ) {
        // for arg in stack.args.iter() {
        //     println!(">> {:?}", arg);
        // }
        // for instr in instrs.iter() {
        //     println!("{}", Instr::with_consts(instr, &consts));
        // }
        let print_log = false;

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

        let end_pc = instr_len as isize -1;
        //println!("lifetimes:");
        let mut sorted_lifetimes: Vec<(isize, isize, u16, bool, i16)> = vec!();
        for (k, v) in &self.lifetimes {
            let id = k;
            let (start, end) = v;
            let end = end.unwrap_or(end_pc);
            let is_input = (*id as usize) < self.size();
            let addr = if is_input {
                let size = block_info.stack_min_size as i16;
                //println!("size {}", size);
                let addr = (*id as isize - size as isize) as i16;
                //let address = (i as isize - size as isize) as i16;
                addr
            } else {
                std::i16::MAX as i16
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
                    if print_log { println!("{}{} has reach end of life, its address @{} is available for writing",
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
                        v.4 = addr;
                    }
                }
            }
            pc += 1;
        }
        //println!("{:?}", bb);

        // patch instruction temporaries with their allocated stack slots
        for instr in instrs {
            for opr in &mut instr.operands {
                match opr {
                    Operand::Temporary { id } => {
                        let res = sorted_lifetimes.iter().find(|&tu| tu.2 == *id);
                        let (_,_,_,_,addr) = res.unwrap();
                        *opr = Operand::Address {
                            offset: *addr
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

        loop {
            let s0 = sets.next();
            let s1 = sets.next();
            match (s0, s1) {
                (Some((dst0, src0)), Some((dst1, src1))) => {
                    instrs.push(Instr::set2(dst0, dst1, src0, src1, imms));
                },
                (Some((dst, src)), None) => {
                    instrs.push(Instr::set2(dst, dst, src, src, imms));
                    break;
                },
                (None, None) => break,
                (None, Some(_)) => unreachable!(),
            }
        }

        if diff != 0 {
            // we need to store in last instruction of the block the stack ptr
            // offset
            if let Some(instr) = instrs.last_mut() {
                // check if last instruction has enough bits left, otherwise we
                // need to push a noop jump instruction for that matter
                instr.sp_offset = diff as i16;
            }
        }
    }
}

pub fn build_super_instructions(bytecode: &[u8], schedule: &Schedule) {
    let mut rom = VmRom::new();
    rom.init(&bytecode, &schedule);
    //
    let opcodes: *const EvmOpcode = bytecode.as_ptr() as *const EvmOpcode;
    let mut stack = StaticStack::new();
    let mut consts: Vec<U256> = Vec::new();
    let mut instrs: Vec<Instr> = Vec::new();
    let mut start_instr = 0;
    let mut super_block_offset = 0;

    let block_infos_len = rom.block_infos_len();
    let mut block_offset: isize = 0;
    assert!(block_infos_len > 0);
    for i in 0..block_infos_len {
        println!("\n==== block #{} ====", i);
        let block_info = rom.get_block_info(i);
        let block_bytes_len = if i < (block_infos_len-1) {
            let next_block_info = rom.get_block_info(i+1);
            next_block_info.start_addr.0 - block_info.start_addr.0
        } else {
            bytecode.len() as u16 - block_info.start_addr.0
        } as isize;
        println!("{:?}", block_info);
        println!("block bytes: {}", block_bytes_len);

        // print block opcodes
        let mut offset: isize = 0;
        while offset < block_bytes_len {
            let opcode = unsafe { *opcodes.offset(block_offset + offset) };
            println!("{:?}", opcode);
            if opcode.is_push() {
                let num_bytes = opcode.push_index() as isize + 1;
                offset += num_bytes;
            }
            offset += 1;
        }
        println!("");

        // build super instructions
        stack.clear(block_info.stack_min_size as usize);
        let block = &bytecode[block_offset as usize..(block_offset + block_bytes_len) as usize];
        stack.eval_block(block, &rom, &mut consts, &mut instrs);

        let block_instr_len = instrs.len() - start_instr;
        stack.alloc_stack_slots(&mut instrs[start_instr..], block_instr_len, &block_info);
        stack.block_fixup(&mut consts, &mut instrs);

        // patch jump addresses
        let mut v = rom.get_bb_info_mut(block_offset as u64);
        v.start_addr.1 = super_block_offset;
        for instr in &instrs[start_instr..] {
            super_block_offset += instr.size() as u16;
        }

        start_instr = instrs.len();

        block_offset += block_bytes_len;
    }

    // compress constants (optional)

    println!("");
    for instr in instrs.iter() {
        let ic = Instr::with_consts(instr, &consts);
        println!("{}", ic);
    }
}