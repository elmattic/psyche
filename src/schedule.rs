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

use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Fork {
    Frontier,
    Thawing,
    Homestead,
    Dao,
    Tangerine,
    Spurious,
    Byzantium,
    Constantinople,
    Istanbul,
    Berlin,
}

const FORK_LEN: usize = Fork::Berlin as usize + 1;

impl FromStr for Fork {
    type Err = ();

    fn from_str(input: &str) -> Result<Fork, Self::Err> {
        match input {
            "Frontier" => Ok(Fork::Frontier),
            "Thawing" => Ok(Fork::Thawing),
            "Homestead" => Ok(Fork::Homestead),
            "Dao" => Ok(Fork::Dao),
            "Tangerine" => Ok(Fork::Tangerine),
            "Spurious" => Ok(Fork::Spurious),
            "Byzantium" => Ok(Fork::Byzantium),
            "Constantinople" => Ok(Fork::Constantinople),
            "Istanbul" => Ok(Fork::Istanbul),
            "Berlin" => Ok(Fork::Berlin),
            _ => Err(()),
        }
    }
}

pub fn to_block_number(fork: Fork) -> u64 {
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
        Fork::Berlin => panic!(),
    }
}

impl Fork {
    pub const fn default() -> Fork {
        Fork::Frontier
    }

    pub fn from_block(number: u64) -> Fork {
        let block_fork = |f| (to_block_number(f), f);
        let block_forks: [(u64, Fork); FORK_LEN] = [
            block_fork(Fork::Frontier),
            block_fork(Fork::Thawing),
            block_fork(Fork::Homestead),
            block_fork(Fork::Dao),
            block_fork(Fork::Tangerine),
            block_fork(Fork::Spurious),
            block_fork(Fork::Byzantium),
            block_fork(Fork::Constantinople),
            block_fork(Fork::Istanbul),
            block_fork(Fork::Istanbul),
        ];
        assert!(number != 0, "block number must be greater than 0");
        let pos = block_forks.iter().position(|(x, _)| *x > number);
        block_forks[pos.unwrap_or(FORK_LEN) - 1].1
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
    pub fork: Fork,
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
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 50, 30, 6, 3, 20], // Berlin
        ];
        Schedule {
            fees: COSTS[fork as usize],
            memory_gas: 3,
            fork,
        }
    }
}
