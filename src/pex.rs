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

use super::opt::{self, BlockInfo, Instr};
use super::schedule::Schedule;
use super::instructions::EvmOpcode;
use super::u256::U256;

/// Experimental multi-tier format for Portable EXecution
pub struct Pex {
    bytes: Vec<u8>,
}

impl Pex {
    /// EIP-170 states a max contract code size of 2**14 + 2**13, we round it
    /// to the next power of two (32 KiB).
    const BYTECODE_SIZE: usize = 32 << 10;

    const HEADER_SIZE: usize = 64;
    const INVALID_DESTS_SIZE: usize = Self::BYTECODE_SIZE / 8;
    const BLOCK_INFO_SIZE: usize = std::mem::size_of::<BlockInfo>();
    const BLOCK_INFOS_SIZE: usize = Self::BYTECODE_SIZE * Self::BLOCK_INFO_SIZE;

    const HEADER_OFFSET: usize = 0;
    const INVALID_DESTS_OFFSET: usize = Self::HEADER_OFFSET + Self::HEADER_SIZE;
    const BLOCK_INFOS_OFFSET: usize = Self::INVALID_DESTS_OFFSET + Self::INVALID_DESTS_SIZE;
    const TEXT_OFFSET: usize = Self::BLOCK_INFOS_OFFSET + Self::BLOCK_INFOS_SIZE;

    pub fn new() -> Pex {
        let mut bytes = Vec::new();
        bytes.resize(Self::TEXT_OFFSET, 0);
        Pex {
            bytes
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn resize(&mut self, new_len: usize) {
        self.bytes.resize(new_len, 0);
    }

    // TODO: return mutable slice instead
    pub fn invalid_dests_ptr(&mut self) -> *mut u8 {
        let offset = Self::INVALID_DESTS_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut u8
        }
    }

    // TODO: return mutable slice instead
    pub fn block_infos_ptr(&mut self) -> *mut BlockInfo {
        let offset = Self::BLOCK_INFOS_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut BlockInfo
        }
    }

    pub fn text_ptr(&mut self) -> *mut u64 {
        let offset = Self::TEXT_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut u64
        }
    }
}

pub fn build(bytecode: &[u8], schedule: &Schedule) -> Pex {
    let mut pex = Pex::new();
    println!("size: {} bytes", pex.len());

    // write invalid destinations
    let invalid_dests_ptr = pex.invalid_dests_ptr();
    let mut i = 0;
    while i < bytecode.len() {
        let opcode = unsafe { std::mem::transmute::<u8, EvmOpcode>(bytecode[i]) };
        if opcode.is_push() {
            let num_bytes = opcode.push_index() + 1;
            let mask: u64 = (1 << num_bytes) - 1;
            let j = (i + 1) as isize;
            let byte_offset = j / 8;
            let bit_offset = j % 8;
            unsafe {
                let ptr = invalid_dests_ptr.offset(byte_offset) as *mut u32;
                *ptr |= (mask as u32) << bit_offset;
            }
            i += num_bytes;
        }
        i += 1;
    }

    let mut block_infos: Vec<BlockInfo> = vec!();
    opt::build_block_infos(bytecode, schedule, &mut block_infos);
    // write block infos
    for (i, bi) in block_infos.iter().enumerate() {
        unsafe {
            let ptr = pex.block_infos_ptr().offset(i as isize);
            *ptr = *bi;
        }
    }

    let mut imms: Vec<U256> = vec!();
    let mut instrs: Vec<Instr> = vec!();
    // write instructions
    opt::build_super_instructions(
        bytecode, &schedule, &mut block_infos, &mut imms, &mut instrs,
    );

    pex
}
