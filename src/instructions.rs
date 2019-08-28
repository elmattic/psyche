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

use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone, FromPrimitive)]
pub enum Instruction {
    STOP,
    ADD,
    MUL,
    SUB,
    SIGNEXTEND,
    GT,
    EQ,
    ISZERO,
    AND,
    OR,
    XOR,
    NOT,
    BYTE,
    SHL,
    POP,
    MLOAD,
    MSTORE,
    MSTORE8,
    JUMP,
    JUMPI,
    PC,
    MSIZE,
    GAS,
    JUMPDEST,
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
    RETURN,
    INVALID
}

#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone, FromPrimitive)]
pub enum EvmInstruction {
    STOP = 0x00,
    ADD = 0x01,
    MUL = 0x02,
    SUB = 0x03,
    SIGNEXTEND = 0x0b,
    GT = 0x11,
    EQ = 0x14,
    ISZERO = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    NOT = 0x19,
    BYTE = 0x1a,
    SHL = 0x1b,
    POP = 0x50,
    MLOAD = 0x51,
    MSTORE = 0x52,
    MSTORE8 = 0x53,
    JUMP = 0x56,
    JUMPI = 0x57,
    PC = 0x58,
    MSIZE = 0x59,
    GAS = 0x5a,
    JUMPDEST = 0x5b,
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
    RETURN = 0xf3,
    INVALID = 0xfe
}

impl Instruction {
    /// Returns true if given instruction is `PUSHN` instruction
    /// PUSH1 -> true
    pub fn is_push(&self) -> bool {
        *self >= Instruction::PUSH1 && *self <= Instruction::PUSH32
    }

    /// Returns the index of the `PUSHN` opcode
    /// PUSH1 -> 0
    pub fn push_index(&self) -> usize {
        ((*self as u8) - (Instruction::PUSH1 as u8)) as usize
    }

    /// Returns the index of the `DUPN` opcode
    /// DUP1 -> 0
    pub fn dup_index(&self) -> usize {
        ((*self as u8) - (Instruction::DUP1 as u8)) as usize
    }

    /// Returns the index of the `SWAPN` opcode
    /// SWAP1 -> 0
    pub fn swap_index(&self) -> usize {
        ((*self as u8) - (Instruction::SWAP1 as u8)) as usize
    }
}

impl EvmInstruction {
    /// Returns true if given instruction is `PUSHN` instruction
    /// PUSH1 -> true
    pub fn is_push(&self) -> bool {
        *self >= EvmInstruction::PUSH1 && *self <= EvmInstruction::PUSH32
    }

    /// Returns true if given instruction is a basic block (BB) terminator
    /// JUMP -> true
    pub fn is_terminator(&self) -> bool {
        match *self {
            EvmInstruction::STOP | EvmInstruction::JUMP |
            EvmInstruction::JUMPI | EvmInstruction::INVALID => true,
            _ => false
        }
    }

    /// Returns the index of the `PUSHN` opcode
    /// PUSH1 -> 0
    pub fn push_index(&self) -> usize {
        ((*self as u8) - (EvmInstruction::PUSH1 as u8)) as usize
    }

    /// Convert to internal representation
    pub fn to_internal(&self) -> Instruction {
        const MAPPING: [Instruction; 256] = [Instruction::STOP, Instruction::ADD, Instruction::MUL, Instruction::SUB, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::SIGNEXTEND, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::GT, Instruction::INVALID, Instruction::INVALID, Instruction::EQ, Instruction::ISZERO, Instruction::AND, Instruction::OR, Instruction::XOR, Instruction::NOT, Instruction::BYTE, Instruction::SHL, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::POP, Instruction::MLOAD, Instruction::MSTORE, Instruction::MSTORE8, Instruction::INVALID, Instruction::INVALID, Instruction::JUMP, Instruction::JUMPI, Instruction::PC, Instruction::MSIZE, Instruction::GAS, Instruction::JUMPDEST, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::PUSH1, Instruction::PUSH2, Instruction::PUSH3, Instruction::PUSH4, Instruction::PUSH5, Instruction::PUSH6, Instruction::PUSH7, Instruction::PUSH8, Instruction::PUSH9, Instruction::PUSH10, Instruction::PUSH11, Instruction::PUSH12, Instruction::PUSH13, Instruction::PUSH14, Instruction::PUSH15, Instruction::PUSH16, Instruction::PUSH17, Instruction::PUSH18, Instruction::PUSH19, Instruction::PUSH20, Instruction::PUSH21, Instruction::PUSH22, Instruction::PUSH23, Instruction::PUSH24, Instruction::PUSH25, Instruction::PUSH26, Instruction::PUSH27, Instruction::PUSH28, Instruction::PUSH29, Instruction::PUSH30, Instruction::PUSH31, Instruction::PUSH32, Instruction::DUP1, Instruction::DUP2, Instruction::DUP3, Instruction::DUP4, Instruction::DUP5, Instruction::DUP6, Instruction::DUP7, Instruction::DUP8, Instruction::DUP9, Instruction::DUP10, Instruction::DUP11, Instruction::DUP12, Instruction::DUP13, Instruction::DUP14, Instruction::DUP15, Instruction::DUP16, Instruction::SWAP1, Instruction::SWAP2, Instruction::SWAP3, Instruction::SWAP4, Instruction::SWAP5, Instruction::SWAP6, Instruction::SWAP7, Instruction::SWAP8, Instruction::SWAP9, Instruction::SWAP10, Instruction::SWAP11, Instruction::SWAP12, Instruction::SWAP13, Instruction::SWAP14, Instruction::SWAP15, Instruction::SWAP16, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::RETURN, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID, Instruction::INVALID];
        MAPPING[*self as usize]
    }
}
