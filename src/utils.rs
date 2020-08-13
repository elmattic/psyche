// Copyright 2020 The Psyche Authors
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

use std::fmt::Write;
use std::num::ParseIntError;

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut temp = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(&mut temp, "{:02x}", b);
    }
    temp
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| {
            let temp: Result<u8, _> = u8::from_str_radix(&s[i..i + 2], 16);
            match temp {
                Ok(_) => temp,
                Err(e) => Err(e)
            }
        })
        .collect()
}

macro_rules! test_feature_bit {
   ($name:ident, $register: ident, $mask:expr) => (
        fn $name() -> bool {
            use core::arch::x86_64::__cpuid;
            #[cfg(target_arch = "x86_64")]
            {
                let result = unsafe { __cpuid(1) };
                return (result.$register & $mask) > 0;
            }
            return false;
        }
    )
}

macro_rules! test_extented_feature_bit {
   ($name:ident, $register: ident, $mask:expr) => (
        fn $name() -> bool {
            use core::arch::x86_64::__cpuid_count;
            #[cfg(target_arch = "x86_64")]
            {
                let result = unsafe { __cpuid_count(7, 0) };
                return (result.$register & $mask) > 0;
            }
            return false;
        }
    )
}

test_feature_bit!(may_i_use_Ssse3, ecx, 1 << 9);
test_extented_feature_bit!(may_i_use_Avx2, ebx, 1 << 5);
test_extented_feature_bit!(may_i_use_Bmi1, ebx, 1 << 3);
test_extented_feature_bit!(may_i_use_Bmi2, ebx, 1 << 8);
test_extented_feature_bit!(may_i_use_Adx, ebx, 1 << 19);
test_extented_feature_bit!(may_i_use_Avx512f, ebx, 1 << 16);

#[allow(unreachable_code)]
pub fn print_config() {
    #[cfg(debug_assertions)]
    {
        println!("mode: debug");
    }
    #[cfg(not(debug_assertions))]
    {
        println!("mode: release");
    }
    let mut features = vec![];
    if may_i_use_Ssse3() { features.push("ssse3"); }
    if may_i_use_Avx2() { features.push("avx2"); }
    if may_i_use_Bmi1() { features.push("bmi1"); }
    if may_i_use_Bmi2() { features.push("bmi2"); }
    if may_i_use_Adx() { features.push("adx"); }
    if may_i_use_Avx512f() { features.push("avx512f"); }
    if features.len() > 0 {
        println!("flags: {}", features.join(" "));
    }
}
