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

use super::opt::BlockInfo;
use super::schedule::Schedule;

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

    pub fn new(bytes: Vec<u8>) -> Pex {
        Pex {
            bytes
        }
    }
}

pub fn build(bytecode: &[u8], schedule: &Schedule) -> Pex {
    let mut bytes: Vec<u8> = vec!();
    const SIZE: usize = 64 << 10; // 64 KiB
    bytes.reserve(SIZE);
    println!("pex: {} bytes", bytes.len());

    Pex::new(bytes)
}
