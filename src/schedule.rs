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

#[derive(Copy, Clone, Debug)]
pub enum Fork {
    Frontier = 0,
    Thawing = 1,
    Homestead = 2,
    Dao = 3,
    Tangerine = 4,
    Spurious = 5,
    Byzantium = 6,
    Constantinople = 7,
    Istanbul = 8,
}

const FORK_LEN: usize = 9;

pub const fn to_block_number(fork: Fork) -> u64 {
    match fork {
        Fork::Frontier => 1,
        Fork::Thawing => 200_000,
        Fork::Homestead => 1_150_000,
        Fork::Dao => 1_920_000,
        Fork::Tangerine => 2_463_000,
        Fork::Spurious => 2_675_000,
        Fork::Byzantium => 4_370_000,
        Fork::Constantinople => 7_280_000,
        Fork::Istanbul => 9_069_000,
    }
}

impl Fork {
    pub const fn default() -> Fork {
        Fork::Frontier
    }

    pub fn from_block(number: u64) -> Fork {
        const BLOCK_FORKS: [(u64, Fork); FORK_LEN] = [
            (to_block_number(Fork::Frontier), Fork::Frontier),
            (to_block_number(Fork::Thawing), Fork::Thawing),
            (to_block_number(Fork::Homestead), Fork::Homestead),
            (to_block_number(Fork::Dao), Fork::Dao),
            (to_block_number(Fork::Tangerine), Fork::Tangerine),
            (to_block_number(Fork::Spurious), Fork::Spurious),
            (to_block_number(Fork::Byzantium), Fork::Byzantium),
            (to_block_number(Fork::Constantinople), Fork::Constantinople),
            (to_block_number(Fork::Istanbul), Fork::Istanbul),
        ];
        assert!(number != 0, "block number must be greater than 0");
        let pos = BLOCK_FORKS.iter().position(|(x, _)| *x > number);
        BLOCK_FORKS[pos.unwrap_or(FORK_LEN) - 1].1
    }
}

#[derive(Copy, Clone)]
pub enum Fee {
    Zero,
    Base,
    VeryLow,
    Low,
    Mid,
    High,
    Balance,
    Jumpdest,
    Exp,
    ExpByte,
    Sha3,
    Sha3Word,
    Copy,
    Blockhash,
}

const FEE_LEN: usize = 14;

impl Fee {
    /// Returns the gas cost associated to a given fork
    pub fn gas(self, schedule: &Schedule) -> u32 {
        schedule.fees[self as usize]
    }
}

#[derive(Debug)]
pub struct Schedule {
    pub fees: [u32; FEE_LEN],
    pub memory_gas: u64,
}

impl Schedule {
    pub fn default() -> Schedule {
        Schedule::from_fork(Fork::default())
    }

    pub fn from_fork(fork: Fork) -> Schedule {
        const COSTS: [[u32; FEE_LEN]; FORK_LEN] = [
            [0, 2, 3, 5, 8, 10, 20, 1, 10, 10, 30, 6, 3, 20], // Frontier
            [0, 2, 3, 5, 8, 10, 20, 1, 10, 10, 30, 6, 3, 20], // Thawing
            [0, 2, 3, 5, 8, 10, 20, 1, 10, 10, 30, 6, 3, 20], // Homestead
            [0, 2, 3, 5, 8, 10, 20, 1, 10, 10, 30, 6, 3, 20], // Dao
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 10, 30, 6, 3, 20], // Tangerine
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 50, 30, 6, 3, 20], // Spurious
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 50, 30, 6, 3, 20], // Byzantium
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 50, 30, 6, 3, 20], // Constantinople
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 50, 30, 6, 3, 20], // Istanbul
        ];
        Schedule {
            fees: COSTS[fork as usize],
            memory_gas: 3,
        }
    }
}
