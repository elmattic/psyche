// Copyright 2019 The Psyche Authors
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

use num_enum::TryFromPrimitive;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    STOP,
    ADD,
    MUL,
    SUB,
    DIV,
    SDIV,
    MOD,
    SMOD,
    ADDMOD,
    MULMOD,
    EXP,
    SIGNEXTEND,
    LT,
    GT,
    SLT,
    SGT,
    EQ,
    ISZERO,
    AND,
    OR,
    XOR,
    NOT,
    BYTE,
    SHL,
    SHR,
    SAR,
    SHA3,
    ADDRESS,
    BALANCE,
    ORIGIN,
    CALLER,
    CALLVALUE,
    CALLDATALOAD,
    CALLDATASIZE,
    CALLDATACOPY,
    CODESIZE,
    CODECOPY,
    GASPRICE,
    EXTCODESIZE,
    EXTCODECOPY,
    RETURNDATASIZE,
    RETURNDATACOPY,
    EXTCODEHASH,
    BLOCKHASH,
    COINBASE,
    TIMESTAMP,
    NUMBER,
    DIFFICULTY,
    GASLIMIT,
    CHAINID,
    SELFBALANCE,
    POP,
    MLOAD,
    MSTORE,
    MSTORE8,
    SLOAD,
    SSTORE,
    JUMP,
    JUMPI,
    PC,
    MSIZE,
    GAS,
    JUMPDEST,
    BEGINSUB,
    RETURNSUB,
    JUMPSUB,
    PUSH1,
    PUSH2,
    PUSH3,
    PUSH4,
    PUSH5,
    PUSH6,
    PUSH7,
    PUSH8,
    PUSH9,
    PUSH10,
    PUSH11,
    PUSH12,
    PUSH13,
    PUSH14,
    PUSH15,
    PUSH16,
    PUSH17,
    PUSH18,
    PUSH19,
    PUSH20,
    PUSH21,
    PUSH22,
    PUSH23,
    PUSH24,
    PUSH25,
    PUSH26,
    PUSH27,
    PUSH28,
    PUSH29,
    PUSH30,
    PUSH31,
    PUSH32,
    DUP1,
    DUP2,
    DUP3,
    DUP4,
    DUP5,
    DUP6,
    DUP7,
    DUP8,
    DUP9,
    DUP10,
    DUP11,
    DUP12,
    DUP13,
    DUP14,
    DUP15,
    DUP16,
    SWAP1,
    SWAP2,
    SWAP3,
    SWAP4,
    SWAP5,
    SWAP6,
    SWAP7,
    SWAP8,
    SWAP9,
    SWAP10,
    SWAP11,
    SWAP12,
    SWAP13,
    SWAP14,
    SWAP15,
    SWAP16,
    LOG0,
    LOG1,
    LOG2,
    LOG3,
    LOG4,
    CREATE,
    CALL,
    CALLCODE,
    RETURN,
    DELEGATECALL,
    CREATE2,
    STATICCALL,
    REVERT,
    INVALID,
    SELFDESTRUCT,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone, FromPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EvmOpcode {
    STOP = 0x00,
    ADD = 0x01,
    MUL = 0x02,
    SUB = 0x03,
    DIV = 0x04,
    SDIV = 0x05,
    MOD = 0x06,
    SMOD = 0x07,
    ADDMOD = 0x08,
    MULMOD = 0x09,
    EXP = 0x0a,
    SIGNEXTEND = 0x0b,
    LT = 0x10,
    GT = 0x11,
    SLT = 0x12,
    SGT = 0x13,
    EQ = 0x14,
    ISZERO = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    NOT = 0x19,
    BYTE = 0x1a,
    SHL = 0x1b,
    SHR = 0x1c,
    SAR = 0x1d,
    SHA3 = 0x20,
    ADDRESS = 0x30,
    BALANCE = 0x31,
    ORIGIN = 0x32,
    CALLER = 0x33,
    CALLVALUE = 0x34,
    CALLDATALOAD = 0x35,
    CALLDATASIZE = 0x36,
    CALLDATACOPY = 0x37,
    CODESIZE = 0x38,
    CODECOPY = 0x39,
    GASPRICE = 0x3a,
    EXTCODESIZE = 0x3b,
    EXTCODECOPY = 0x3c,
    RETURNDATASIZE = 0x3d,
    RETURNDATACOPY = 0x3e,
    EXTCODEHASH = 0x3f,
    BLOCKHASH = 0x40,
    COINBASE = 0x41,
    TIMESTAMP = 0x42,
    NUMBER = 0x43,
    DIFFICULTY = 0x44,
    GASLIMIT = 0x45,
    CHAINID = 0x46,
    SELFBALANCE = 0x47,
    POP = 0x50,
    MLOAD = 0x51,
    MSTORE = 0x52,
    MSTORE8 = 0x53,
    SLOAD = 0x54,
    SSTORE = 0x55,
    JUMP = 0x56,
    JUMPI = 0x57,
    PC = 0x58,
    MSIZE = 0x59,
    GAS = 0x5a,
    JUMPDEST = 0x5b,
    BEGINSUB = 0x5c,
    RETURNSUB = 0x5d,
    JUMPSUB = 0x5e,
    PUSH1 = 0x60,
    PUSH2 = 0x61,
    PUSH3 = 0x62,
    PUSH4 = 0x63,
    PUSH5 = 0x64,
    PUSH6 = 0x65,
    PUSH7 = 0x66,
    PUSH8 = 0x67,
    PUSH9 = 0x68,
    PUSH10 = 0x69,
    PUSH11 = 0x6a,
    PUSH12 = 0x6b,
    PUSH13 = 0x6c,
    PUSH14 = 0x6d,
    PUSH15 = 0x6e,
    PUSH16 = 0x6f,
    PUSH17 = 0x70,
    PUSH18 = 0x71,
    PUSH19 = 0x72,
    PUSH20 = 0x73,
    PUSH21 = 0x74,
    PUSH22 = 0x75,
    PUSH23 = 0x76,
    PUSH24 = 0x77,
    PUSH25 = 0x78,
    PUSH26 = 0x79,
    PUSH27 = 0x7a,
    PUSH28 = 0x7b,
    PUSH29 = 0x7c,
    PUSH30 = 0x7d,
    PUSH31 = 0x7e,
    PUSH32 = 0x7f,
    DUP1 = 0x80,
    DUP2 = 0x81,
    DUP3 = 0x82,
    DUP4 = 0x83,
    DUP5 = 0x84,
    DUP6 = 0x85,
    DUP7 = 0x86,
    DUP8 = 0x87,
    DUP9 = 0x88,
    DUP10 = 0x89,
    DUP11 = 0x8a,
    DUP12 = 0x8b,
    DUP13 = 0x8c,
    DUP14 = 0x8d,
    DUP15 = 0x8e,
    DUP16 = 0x8f,
    SWAP1 = 0x90,
    SWAP2 = 0x91,
    SWAP3 = 0x92,
    SWAP4 = 0x93,
    SWAP5 = 0x94,
    SWAP6 = 0x95,
    SWAP7 = 0x96,
    SWAP8 = 0x97,
    SWAP9 = 0x98,
    SWAP10 = 0x99,
    SWAP11 = 0x9a,
    SWAP12 = 0x9b,
    SWAP13 = 0x9c,
    SWAP14 = 0x9d,
    SWAP15 = 0x9e,
    SWAP16 = 0x9f,
    LOG0 = 0xa0,
    LOG1 = 0xa1,
    LOG2 = 0xa2,
    LOG3 = 0xa3,
    LOG4 = 0xa4,
    CREATE = 0xf0,
    CALL = 0xf1,
    CALLCODE = 0xf2,
    RETURN = 0xf3,
    DELEGATECALL = 0xf4,
    CREATE2 = 0xf5,
    STATICCALL = 0xfa,
    REVERT = 0xfd,
    INVALID = 0xfe,
    SELFDESTRUCT = 0xff
}

use std::fmt;

pub enum EvmInstruction<'a> {
    SingleByte { addr: usize, opcode: EvmOpcode },
    MultiByte { addr: usize, opcode: EvmOpcode, bytes: &'a[u8] },
}

impl fmt::Display for EvmOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Opcode {
    /// Returns true if given instruction is `PUSHN` instruction
    /// PUSH1 -> true
    pub fn is_push(&self) -> bool {
        *self >= Opcode::PUSH1 && *self <= Opcode::PUSH32
    }

    /// Returns the index of the `PUSHN` opcode
    /// PUSH1 -> 0
    pub fn push_index(&self) -> usize {
        ((*self as u8) - (Opcode::PUSH1 as u8)) as usize
    }

    /// Returns the index of the `DUPN` opcode
    /// DUP1 -> 0
    pub fn dup_index(&self) -> usize {
        ((*self as u8) - (Opcode::DUP1 as u8)) as usize
    }

    /// Returns the index of the `SWAPN` opcode
    /// SWAP1 -> 0
    pub fn swap_index(&self) -> usize {
        ((*self as u8) - (Opcode::SWAP1 as u8)) as usize
    }
}

impl EvmOpcode {
    /// Returns true if given opcode is `PUSHN` opcode
    /// PUSH1 -> true
    pub fn is_push(&self) -> bool {
        *self >= EvmOpcode::PUSH1 && *self <= EvmOpcode::PUSH32
    }

    /// Returns true if given opcode is a basic block (BB) terminator
    /// JUMP -> true
    pub fn is_terminator(&self) -> bool {
        match *self {
            EvmOpcode::STOP | EvmOpcode::JUMP |
            EvmOpcode::JUMPI | EvmOpcode::INVALID | EvmOpcode::GAS => true,
            _ => false
        }
    }

    /// Returns the index of the `PUSHN` opcode
    /// PUSH1 -> 0
    pub fn push_index(&self) -> usize {
        ((*self as u8) - (EvmOpcode::PUSH1 as u8)) as usize
    }

    /// Convert to internal representation
    pub fn to_internal(&self) -> Opcode {
        const MAPPING: [Opcode; 256] = [Opcode::STOP, Opcode::ADD, Opcode::MUL, Opcode::SUB, Opcode::DIV, Opcode::SDIV, Opcode::MOD, Opcode::SMOD, Opcode::ADDMOD, Opcode::MULMOD, Opcode::EXP, Opcode::SIGNEXTEND, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::LT, Opcode::GT, Opcode::SLT, Opcode::SGT, Opcode::EQ, Opcode::ISZERO, Opcode::AND, Opcode::OR, Opcode::XOR, Opcode::NOT, Opcode::BYTE, Opcode::SHL, Opcode::SHR, Opcode::SAR, Opcode::INVALID, Opcode::INVALID, Opcode::SHA3, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::ADDRESS, Opcode::BALANCE, Opcode::ORIGIN, Opcode::CALLER, Opcode::CALLVALUE, Opcode::CALLDATALOAD, Opcode::CALLDATASIZE, Opcode::CALLDATACOPY, Opcode::CODESIZE, Opcode::CODECOPY, Opcode::GASPRICE, Opcode::EXTCODESIZE, Opcode::EXTCODECOPY, Opcode::RETURNDATASIZE, Opcode::RETURNDATACOPY, Opcode::EXTCODEHASH, Opcode::BLOCKHASH, Opcode::COINBASE, Opcode::TIMESTAMP, Opcode::NUMBER, Opcode::DIFFICULTY, Opcode::GASLIMIT, Opcode::CHAINID, Opcode::SELFBALANCE, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::POP, Opcode::MLOAD, Opcode::MSTORE, Opcode::MSTORE8, Opcode::SLOAD, Opcode::SSTORE, Opcode::JUMP, Opcode::JUMPI, Opcode::PC, Opcode::MSIZE, Opcode::GAS, Opcode::JUMPDEST, Opcode::BEGINSUB, Opcode::RETURNSUB, Opcode::JUMPSUB, Opcode::INVALID, Opcode::PUSH1, Opcode::PUSH2, Opcode::PUSH3, Opcode::PUSH4, Opcode::PUSH5, Opcode::PUSH6, Opcode::PUSH7, Opcode::PUSH8, Opcode::PUSH9, Opcode::PUSH10, Opcode::PUSH11, Opcode::PUSH12, Opcode::PUSH13, Opcode::PUSH14, Opcode::PUSH15, Opcode::PUSH16, Opcode::PUSH17, Opcode::PUSH18, Opcode::PUSH19, Opcode::PUSH20, Opcode::PUSH21, Opcode::PUSH22, Opcode::PUSH23, Opcode::PUSH24, Opcode::PUSH25, Opcode::PUSH26, Opcode::PUSH27, Opcode::PUSH28, Opcode::PUSH29, Opcode::PUSH30, Opcode::PUSH31, Opcode::PUSH32, Opcode::DUP1, Opcode::DUP2, Opcode::DUP3, Opcode::DUP4, Opcode::DUP5, Opcode::DUP6, Opcode::DUP7, Opcode::DUP8, Opcode::DUP9, Opcode::DUP10, Opcode::DUP11, Opcode::DUP12, Opcode::DUP13, Opcode::DUP14, Opcode::DUP15, Opcode::DUP16, Opcode::SWAP1, Opcode::SWAP2, Opcode::SWAP3, Opcode::SWAP4, Opcode::SWAP5, Opcode::SWAP6, Opcode::SWAP7, Opcode::SWAP8, Opcode::SWAP9, Opcode::SWAP10, Opcode::SWAP11, Opcode::SWAP12, Opcode::SWAP13, Opcode::SWAP14, Opcode::SWAP15, Opcode::SWAP16, Opcode::LOG0, Opcode::LOG1, Opcode::LOG2, Opcode::LOG3, Opcode::LOG4, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::CREATE, Opcode::CALL, Opcode::CALLCODE, Opcode::RETURN, Opcode::DELEGATECALL, Opcode::CREATE2, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::INVALID, Opcode::STATICCALL, Opcode::INVALID, Opcode::INVALID, Opcode::REVERT, Opcode::INVALID, Opcode::SELFDESTRUCT];
        MAPPING[*self as usize]
    }

    pub fn iter() -> std::slice::Iter<'static, EvmOpcode> {
        const VALUES: [EvmOpcode; 145] = [EvmOpcode::STOP, EvmOpcode::ADD, EvmOpcode::MUL, EvmOpcode::SUB, EvmOpcode::DIV, EvmOpcode::SDIV, EvmOpcode::MOD, EvmOpcode::SMOD, EvmOpcode::ADDMOD, EvmOpcode::MULMOD, EvmOpcode::EXP, EvmOpcode::SIGNEXTEND, EvmOpcode::LT, EvmOpcode::GT, EvmOpcode::SLT, EvmOpcode::SGT, EvmOpcode::EQ, EvmOpcode::ISZERO, EvmOpcode::AND, EvmOpcode::OR, EvmOpcode::XOR, EvmOpcode::NOT, EvmOpcode::BYTE, EvmOpcode::SHL, EvmOpcode::SHR, EvmOpcode::SAR, EvmOpcode::SHA3, EvmOpcode::ADDRESS, EvmOpcode::BALANCE, EvmOpcode::ORIGIN, EvmOpcode::CALLER, EvmOpcode::CALLVALUE, EvmOpcode::CALLDATALOAD, EvmOpcode::CALLDATASIZE, EvmOpcode::CALLDATACOPY, EvmOpcode::CODESIZE, EvmOpcode::CODECOPY, EvmOpcode::GASPRICE, EvmOpcode::EXTCODESIZE, EvmOpcode::EXTCODECOPY, EvmOpcode::RETURNDATASIZE, EvmOpcode::RETURNDATACOPY, EvmOpcode::EXTCODEHASH, EvmOpcode::BLOCKHASH, EvmOpcode::COINBASE, EvmOpcode::TIMESTAMP, EvmOpcode::NUMBER, EvmOpcode::DIFFICULTY, EvmOpcode::GASLIMIT, EvmOpcode::CHAINID, EvmOpcode::SELFBALANCE, EvmOpcode::POP, EvmOpcode::MLOAD, EvmOpcode::MSTORE, EvmOpcode::MSTORE8, EvmOpcode::SLOAD, EvmOpcode::SSTORE, EvmOpcode::JUMP, EvmOpcode::JUMPI, EvmOpcode::PC, EvmOpcode::MSIZE, EvmOpcode::GAS, EvmOpcode::JUMPDEST, EvmOpcode::BEGINSUB, EvmOpcode::RETURNSUB, EvmOpcode::JUMPSUB, EvmOpcode::PUSH1, EvmOpcode::PUSH2, EvmOpcode::PUSH3, EvmOpcode::PUSH4, EvmOpcode::PUSH5, EvmOpcode::PUSH6, EvmOpcode::PUSH7, EvmOpcode::PUSH8, EvmOpcode::PUSH9, EvmOpcode::PUSH10, EvmOpcode::PUSH11, EvmOpcode::PUSH12, EvmOpcode::PUSH13, EvmOpcode::PUSH14, EvmOpcode::PUSH15, EvmOpcode::PUSH16, EvmOpcode::PUSH17, EvmOpcode::PUSH18, EvmOpcode::PUSH19, EvmOpcode::PUSH20, EvmOpcode::PUSH21, EvmOpcode::PUSH22, EvmOpcode::PUSH23, EvmOpcode::PUSH24, EvmOpcode::PUSH25, EvmOpcode::PUSH26, EvmOpcode::PUSH27, EvmOpcode::PUSH28, EvmOpcode::PUSH29, EvmOpcode::PUSH30, EvmOpcode::PUSH31, EvmOpcode::PUSH32, EvmOpcode::DUP1, EvmOpcode::DUP2, EvmOpcode::DUP3, EvmOpcode::DUP4, EvmOpcode::DUP5, EvmOpcode::DUP6, EvmOpcode::DUP7, EvmOpcode::DUP8, EvmOpcode::DUP9, EvmOpcode::DUP10, EvmOpcode::DUP11, EvmOpcode::DUP12, EvmOpcode::DUP13, EvmOpcode::DUP14, EvmOpcode::DUP15, EvmOpcode::DUP16, EvmOpcode::SWAP1, EvmOpcode::SWAP2, EvmOpcode::SWAP3, EvmOpcode::SWAP4, EvmOpcode::SWAP5, EvmOpcode::SWAP6, EvmOpcode::SWAP7, EvmOpcode::SWAP8, EvmOpcode::SWAP9, EvmOpcode::SWAP10, EvmOpcode::SWAP11, EvmOpcode::SWAP12, EvmOpcode::SWAP13, EvmOpcode::SWAP14, EvmOpcode::SWAP15, EvmOpcode::SWAP16, EvmOpcode::LOG0, EvmOpcode::LOG1, EvmOpcode::LOG2, EvmOpcode::LOG3, EvmOpcode::LOG4, EvmOpcode::CREATE, EvmOpcode::CALL, EvmOpcode::CALLCODE, EvmOpcode::RETURN, EvmOpcode::DELEGATECALL, EvmOpcode::CREATE2, EvmOpcode::STATICCALL, EvmOpcode::REVERT, EvmOpcode::INVALID, EvmOpcode::SELFDESTRUCT];
        VALUES.iter()
    }
}
