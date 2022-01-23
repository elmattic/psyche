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
use super::u256::U256;

/// Experimental multi-tier format for Portable EXecution
pub struct Pex {
    bytes: Vec<u8>,
}

impl Pex {
    /// EIP-170 states a max contract code size of 2**14 + 2**13, we round it
    /// to the next power of two (32 KiB).
    pub const BYTECODE_SIZE: usize = 32 << 10;

    const HEADER_SIZE: usize = 64;
    const VALID_JUMPDESTS_SIZE: usize = Self::BYTECODE_SIZE / 8;
    const BLOCK_INFOS_SIZE: usize = Self::BYTECODE_SIZE * std::mem::size_of::<BlockInfo>();
    const IMMS_SIZE: usize = (16 << 10) * 32;
    const TEXT_SIZE: usize = Self::BYTECODE_SIZE * 16;

    const HEADER_OFFSET: usize = 0;
    const VALID_JUMPDESTS_OFFSET: usize = Self::HEADER_OFFSET + Self::HEADER_SIZE;
    const BLOCK_INFOS_OFFSET: usize = Self::VALID_JUMPDESTS_OFFSET + Self::VALID_JUMPDESTS_SIZE;
    const IMMS_OFFSET: usize = Self::BLOCK_INFOS_OFFSET + Self::BLOCK_INFOS_SIZE;
    const TEXT_OFFSET: usize = Self::IMMS_OFFSET + Self::IMMS_SIZE;

    pub fn new() -> Pex {
        let mut bytes = Vec::new();
        bytes.resize(Self::TEXT_OFFSET+Self::TEXT_SIZE, 0);
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

    pub fn valid_jumpdests_mut(&mut self) -> *mut u64 {
        let offset = Self::VALID_JUMPDESTS_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut u64
        }
    }

    pub fn valid_jumpdests(&self) -> *const u64 {
        let offset = Self::VALID_JUMPDESTS_OFFSET as isize;
        unsafe {
            self.bytes.as_ptr().offset(offset) as *mut u64
        }
    }

    pub fn block_infos_ptr(&mut self) -> *mut BlockInfo {
        let offset = Self::BLOCK_INFOS_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut BlockInfo
        }
    }

    pub fn imms_ptr_mut(&mut self) -> *mut U256 {
        let offset = Self::TEXT_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut U256
        }
    }

    pub fn text_ptr_mut(&mut self) -> *mut u64 {
        let offset = Self::TEXT_OFFSET as isize;
        unsafe {
            self.bytes.as_mut_ptr().offset(offset) as *mut u64
        }
    }

    pub fn is_jumpdest(&self, addr: u32) -> bool {
        let addr = (addr as isize) % (Self::BYTECODE_SIZE as isize);
        let bits = unsafe { *self.valid_jumpdests().offset(addr % 64) };
        let mask = 1 << (addr % 64);
        (bits & mask) > 0
    }
}

pub fn build(bytecode: &[u8], schedule: &Schedule) -> Pex {
    let mut pex = Pex::new();
    println!("pex size: {} bytes", pex.len());

    opt::build_valid_jumpdests(bytecode, pex.valid_jumpdests_mut());

    let mut block_infos: Vec<BlockInfo> = vec!();
    opt::build_block_infos(bytecode, schedule, &mut block_infos);

    let mut imms: Vec<U256> = vec!();
    let mut instrs: Vec<Instr> = vec!();
    opt::build_super_instructions(
        bytecode, pex.valid_jumpdests(), &mut block_infos, &mut imms, &mut instrs,
    );

    for (i, bi) in block_infos.iter().enumerate() {
        unsafe {
            let ptr = pex.block_infos_ptr().offset(i as isize);
            *ptr = *bi;
        }
    }

    for (i, imm) in imms.iter().enumerate() {
        unsafe {
            let ptr = pex.imms_ptr_mut().offset(i as isize);
            *ptr = *imm;
        }
    }

    let mut ptr: *mut u64 = pex.text_ptr_mut();
    for instr in &instrs {
        let len = instr.encode(ptr);
        ptr = unsafe { ptr.offset(len as isize) };
    }

    pex
}
