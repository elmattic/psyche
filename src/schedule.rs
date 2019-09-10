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

#[derive(Copy, Clone)]
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

impl Fork {
    pub fn default() -> Fork {
        Fork::Frontier
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
    Sha3,
    Copy,
    Blockhash,
}

impl Fee {
    /// Returns the gas cost associated to a given fork
    pub fn gas(self, fork: Fork) -> u32 {
        const COSTS: [[u32; 12]; 9] = [
            [0, 2, 3, 5, 8, 10,  20, 1, 10, 30, 3, 20], // Frontier
            [0, 2, 3, 5, 8, 10,  20, 1, 10, 30, 3, 20], // Thawing
            [0, 2, 3, 5, 8, 10,  20, 1, 10, 30, 3, 20], // Homestead
            [0, 2, 3, 5, 8, 10,  20, 1, 10, 30, 3, 20], // Dao
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 30, 3, 20], // Tangerine
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 30, 3, 20], // Spurious
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 30, 3, 20], // Byzantium
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 30, 3, 20], // Constantinople
            [0, 2, 3, 5, 8, 10, 400, 1, 10, 30, 3, 20], // Istanbul
        ];
        COSTS[fork as usize][self as usize]
    }
 }
