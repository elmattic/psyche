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

pub use self::Instruction::*;

#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone, FromPrimitive)]
pub enum Instruction {
    STOP = 0x00,
    SIGNEXTEND = 0x0b,
    EQ = 0x14,
    ISZERO = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    NOT = 0x19,
    BYTE = 0x1a,
    SHL = 0x1b,
    POP = 0x50,
    JUMP = 0x56,
    JUMPI = 0x57,
    PC = 0x58,
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
    DUP1  = 0x80,
    DUP2  = 0x81,
    DUP3  = 0x82,
    DUP4  = 0x83,
    SWAP1 = 0x90,
    INVALID = 0x92
}

impl Instruction {
    /// Returns true if given instruction is `PUSHN` instruction
    /// PUSH1 -> true
    pub fn is_push(&self) -> bool {
        *self >= PUSH1 && *self <= PUSH32
    }

    /// Returns number of bytes to read for `PUSHN` instruction
    /// PUSH1 -> 1
    pub fn push_bytes(&self) -> usize {
        ((*self as u8) - (PUSH1 as u8) + 1) as usize
    }

    /// Returns stack position of item to duplicate
    /// DUP1 -> 0
    pub fn dup_position(&self) -> usize {
        ((*self as u8) - (DUP1 as u8)) as usize
    }

    /// Returns stack position of item to SWAP top with
    /// SWAP1 -> 1
    pub fn swap_position(&self) -> usize {
        ((*self as u8) - (SWAP1 as u8) + 1) as usize
    }
}
