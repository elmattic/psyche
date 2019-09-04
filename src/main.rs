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

//#![feature(asm)]

extern crate clap;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod instructions;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use clap::{Arg, App, SubCommand};
use num_traits::FromPrimitive;
use std::convert::TryFrom;
use std::env;
use std::fmt;
use std::{fmt::Write, num::ParseIntError};
use instructions::{EvmOpcode, EvmInstruction, Opcode};
use instructions::Opcode::*;

#[repr(align(32))]
#[derive(Copy, Clone)]
struct U256(pub [u64; 4]);

impl U256 {
    pub fn from_u64(value: u64) -> U256 {
        return U256 { 0: [value, 0, 0, 0] };
    }

    pub fn low_u64(&self) -> u64 {
        return self.0[0];
    }
}

trait __m256iExt {
    unsafe fn as_u256(&self) -> U256;
}

impl __m256iExt for __m256i {
    unsafe fn as_u256(&self) -> U256 {
        return std::mem::transmute::<__m256i, U256>(*self);
    }
}

#[derive(Debug)]
pub enum VmError {
    None,
    StackUnderflow,
    StackOverflow,
    OutOfGas,
    InvalidJumpDest,
    InvalidInstruction,
}

#[allow(unreachable_code)]
unsafe fn load_u256(src: *const U256, offset: isize) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let src = src.offset(offset) as *const __m256i;
        let result = _mm256_load_si256(src);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let src = src.offset(offset) as *const __m128i;
        let result = (_mm_load_si128(src), _mm_load_si128(src.offset(1)));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    return *src;
}

#[allow(unreachable_code)]
unsafe fn loadu_u256(src: *const U256, offset: isize) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let src = src.offset(offset) as *const __m256i;
        let result = _mm256_loadu_si256(src);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let src = src.offset(offset) as *const __m128i;
        let result = (_mm_loadu_si128(src), _mm_loadu_si128(src.offset(1)));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    return *src;
}

#[allow(unreachable_code)]
unsafe fn store_u256(dest: *mut U256, value: U256, offset: isize) {
    #[cfg(target_feature = "avx2")]
    {
        let value = std::mem::transmute::<U256, __m256i>(value);
        let dest = dest.offset(offset) as *mut __m256i;
        _mm256_store_si256(dest, value);
        return;
    }
    #[cfg(target_feature = "ssse3")]
    {
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let dest = dest.offset(offset) as *mut __m128i;
        _mm_store_si128(dest, value.0);
        _mm_store_si128(dest.offset(1), value.1);
        return;
    }
    *dest = value;
}

#[allow(unreachable_code)]
unsafe fn storeu_u256(dest: *mut U256, value: U256, offset: isize) {
    #[cfg(target_feature = "avx2")]
    {
        let value = std::mem::transmute::<U256, __m256i>(value);
        let dest = dest.offset(offset) as *mut __m256i;
        _mm256_storeu_si256(dest, value);
        return;
    }
    #[cfg(target_feature = "ssse3")]
    {
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let dest = dest.offset(offset) as *mut __m128i;
        _mm_storeu_si128(dest, value.0);
        _mm_storeu_si128(dest.offset(1), value.1);
        return;
    }
    *dest = value;
}

#[allow(unreachable_code)]
unsafe fn load16_u256(src: *const U256, num_bytes: i32) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane8_id = _mm256_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let src = src as *const __m128i;
        let value = _mm256_zextsi128_si256(_mm_loadu_si128(src));
        let sfloor = _mm_set_epi32(0, 0, 0, (255 - 32) + num_bytes);
        let floor = _mm256_broadcastb_epi8(sfloor);
        let ssum = _mm256_adds_epu8(lane8_id, floor);
        let mask = _mm256_cmpeq_epi8(ssum, all_ones);
        return std::mem::transmute::<__m256i, U256>(_mm256_and_si256(value, mask));
    }
    #[cfg(target_feature = "ssse3")]
    {
        let lane8_id = _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let all_ones = _mm_set_epi64x(-1, -1);
        let zero = _mm_setzero_si128();
        //
        let src = src as *const __m128i;
        let value = _mm_loadu_si128(src);
        let sfloor = _mm_set_epi32(0, 0, 0, (255 - 16) + num_bytes);
        let floor = _mm_shuffle_epi8(sfloor, zero);
        let ssum = _mm_adds_epu8(lane8_id, floor);
        let mask = _mm_cmpeq_epi8(ssum, all_ones);
        return std::mem::transmute::<(__m128i, __m128i), U256>((_mm_and_si128(value, mask), zero));
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn load32_u256(src: *const U256, num_bytes: i32) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane8_id = _mm256_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let src = src as *const __m256i;
        let value = _mm256_loadu_si256(src);
        let sfloor = _mm_set_epi32(0, 0, 0, (255 - 32) + num_bytes);
        let floor = _mm256_broadcastb_epi8(sfloor);
        let ssum = _mm256_adds_epu8(lane8_id, floor);
        let mask = _mm256_cmpeq_epi8(ssum, all_ones);
        return std::mem::transmute::<__m256i, U256>(_mm256_and_si256(value, mask));
    }
    #[cfg(target_feature = "ssse3")]
    {
        let lane8_id = _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let all_ones = _mm_set_epi64x(-1, -1);
        //
        let src = src as *const __m128i;
        let valuelo = _mm_loadu_si128(src);
        let valuehi = _mm_loadu_si128(src.offset(1));
        let sfloor = _mm_set_epi32(0, 0, 0, (255 - 32) + num_bytes);
        let floor = _mm_shuffle_epi8(sfloor, _mm_setzero_si128());
        let ssum = _mm_adds_epu8(lane8_id, floor);
        let mask = _mm_cmpeq_epi8(ssum, all_ones);
        return std::mem::transmute::<(__m128i, __m128i), U256>((valuelo, _mm_and_si128(valuehi, mask)));
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn bswap_u256(value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane8_id = _mm256_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
        const SWAP_LANE128: i32 = (1 << 0) + (0 << 4);
        //
        let value = std::mem::transmute::<U256, __m256i>(value);
        let bswap = _mm256_shuffle_epi8(value, lane8_id);
        let result = _mm256_permute2x128_si256(bswap, bswap, SWAP_LANE128);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let lane8_id = _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        //
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let resultlo = _mm_shuffle_epi8(value.1, lane8_id);
        let resulthi = _mm_shuffle_epi8(value.0, lane8_id);
        return std::mem::transmute::<(__m128i, __m128i), U256>((resultlo, resulthi));
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn is_zero_u256(value: U256) -> bool {
    #[cfg(target_feature = "avx2")]
    {
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let value = std::mem::transmute::<U256, __m256i>(value);
        let zf = _mm256_testz_si256(all_ones, value);
        return zf != 0;
    }
    #[cfg(target_feature = "ssse3")]
    {
        let zero = _mm_setzero_si128();
        //
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let masklo = _mm_cmpeq_epi32(value.0, zero);
        let maskhi = _mm_cmpeq_epi32(value.1, zero);
        let mask16 = _mm_movemask_epi8(_mm_and_si128(masklo, maskhi));
        return mask16 == 0xffff;
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn is_ltpow2_u256(value: U256, pow2: usize) -> bool {
    #[cfg(target_feature = "avx2")]
    {
        let one = _mm256_set_epi64x(0, 0, 0, 1);
        //
        let value = std::mem::transmute::<U256, __m256i>(value);
        let mask = _mm256_sub_epi64(_mm256_set_epi64x(0, 0, 0, pow2 as i64), one);
        let hipart = _mm256_andnot_si256(mask, value);
        let temp = std::mem::transmute::<__m256i, U256>(hipart);
        let result = is_zero_u256(temp);
        return result;
    }
    #[cfg(target_feature = "ssse3")]
    {
        let one = _mm_set_epi64x(0, 1);
        //
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let mask = _mm_sub_epi64(_mm_set_epi64x(0, pow2 as i64), one);
        let hipart = _mm_andnot_si128(mask, value.0);
        let temp = std::mem::transmute::<(__m128i, __m128i), U256>((hipart, value.1));
        let result = is_zero_u256(temp);
        return result;
    }
    unimplemented!()
}

unsafe fn broadcast_avx2(value: bool) -> __m256i {
    let mask = _mm_set_epi32(0, 0, 0, if value { -1 } else { 0 });
    return _mm256_broadcastd_epi32(mask);
}

unsafe fn broadcast_sse2(value: bool) -> __m128i {
    let mask = _mm_set_epi32(0, 0, 0, if value { -1 } else { 0 });
    return _mm_shuffle_epi32(mask, 0);
}

#[allow(unreachable_code)]
unsafe fn mm_blendv_epi8(a: __m128i, b: __m128i, mask: __m128i) -> __m128i {
    #[cfg(target_feature = "sse4.1")]
    {
        return _mm_blendv_epi8(a, b, mask);
    }
    return _mm_or_si128(_mm_and_si128(b, mask), _mm_andnot_si128(mask, a));
}

#[allow(unreachable_code)]
unsafe fn signextend_u256(a: U256, b: U256, value: i64) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let one = _mm256_set_epi64x(0, 0, 0, 1);
        let lane8_id = _mm256_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let _a = std::mem::transmute::<U256, __m256i>(a);
        let _b = std::mem::transmute::<U256, __m256i>(b);
        let signbit = _mm_srli_epi16(_mm_set_epi64x(0, value), 7);
        let signmask8 = _mm_cmpeq_epi8(signbit, _mm256_castsi256_si128(one));
        let signmask = _mm256_broadcastb_epi8(signmask8);
        let alo = _mm256_castsi256_si128(_a);
        let sfloor = _mm_add_epi8(_mm_set_epi64x(0, 255 - 31), alo);
        let floor = _mm256_broadcastb_epi8(sfloor);
        let ssum = _mm256_adds_epu8(lane8_id, floor);
        let mask = _mm256_cmpeq_epi8(ssum, all_ones);
        let temp = _mm256_blendv_epi8(signmask, _b, mask);
        let lt32 = broadcast_avx2(is_ltpow2_u256(a, 32));
        let result = _mm256_blendv_epi8(_b, temp, lt32);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let zero = _mm_setzero_si128();
        let one = _mm_set_epi64x(0, 1);
        let lane8_id = _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let all_ones = _mm_set_epi64x(-1, -1);
        //
        let _a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let _b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let signbit = _mm_srli_epi16(_mm_set_epi64x(0, value), 7);
        let signmask8 = _mm_cmpeq_epi8(signbit, one);
        let signmask = _mm_shuffle_epi8(signmask8, zero);
        let sfloorlo = _mm_adds_epu8(_mm_set_epi64x(0, 255 - 15), _a.0);
        let floorlo = _mm_shuffle_epi8(sfloorlo, zero);
        let ssumlo = _mm_adds_epu8(lane8_id, floorlo);
        let masklo = _mm_cmpeq_epi8(ssumlo, all_ones);
        let templo = mm_blendv_epi8(signmask, _b.0, masklo);
        let sfloorhi = _mm_add_epi8(_mm_set_epi64x(0, 255 - 31), _a.0);
        let floorhi = _mm_shuffle_epi8(sfloorhi, zero);
        let ssumhi = _mm_adds_epu8(lane8_id, floorhi);
        let maskhi = _mm_cmpeq_epi8(ssumhi, all_ones);
        let temphi = mm_blendv_epi8(signmask, _b.1, maskhi);
        let lt32 = broadcast_sse2(is_ltpow2_u256(a, 32));
        let resultlo = mm_blendv_epi8(_b.0, templo, lt32);
        let resulthi = mm_blendv_epi8(_b.1, temphi, lt32);
        return std::mem::transmute::<(__m128i, __m128i), U256>((resultlo, resulthi));
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn eq_u256(a: U256, b: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let a = std::mem::transmute::<U256, __m256i>(a);
        let b = std::mem::transmute::<U256, __m256i>(b);
        let mask = _mm256_cmpeq_epi8(a, b);
        let cf = _mm256_testc_si256(mask, all_ones);
        let result = _mm256_set_epi64x(0, 0, 0, cf as i64);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let masklo = _mm_cmpeq_epi8(a.0, b.0);
        let maskhi = _mm_cmpeq_epi8(a.1, b.1);
        let mask16 = _mm_movemask_epi8(_mm_and_si128(masklo, maskhi));
        let bit = (mask16 == 0xffff) as i64;
        let result = (_mm_set_epi64x(0, bit), _mm_setzero_si128());
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn iszero_u256(a: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let bit = is_zero_u256(a) as i64;
        let result = _mm256_set_epi64x(0, 0, 0, bit);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let bit = is_zero_u256(a) as i64;
        let result = (_mm_set_epi64x(0, bit), _mm_setzero_si128());
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn and_u256(a: U256, b: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let a = std::mem::transmute::<U256, __m256i>(a);
        let b = std::mem::transmute::<U256, __m256i>(b);
        let result = _mm256_and_si256(a, b);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let result = (_mm_and_si128(a.0, b.0), _mm_and_si128(a.1, b.1));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn or_u256(a: U256, b: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let a = std::mem::transmute::<U256, __m256i>(a);
        let b = std::mem::transmute::<U256, __m256i>(b);
        let result = _mm256_or_si256(a, b);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let result = (_mm_or_si128(a.0, b.0), _mm_or_si128(a.1, b.1));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn xor_u256(a: U256, b: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let a = std::mem::transmute::<U256, __m256i>(a);
        let b = std::mem::transmute::<U256, __m256i>(b);
        let result = _mm256_xor_si256(a, b);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let result = (_mm_xor_si128(a.0, b.0), _mm_xor_si128(a.1, b.1));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    unimplemented!()
}

#[allow(unreachable_code)]
unsafe fn not_u256(value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let value = std::mem::transmute::<U256, __m256i>(value);
        let result = _mm256_andnot_si256(value, all_ones);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let all_ones = _mm_set_epi64x(-1, -1);
        //
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let resultlo = _mm_andnot_si128(value.0, all_ones);
        let resulthi = _mm_andnot_si128(value.1, all_ones);
        return std::mem::transmute::<(__m128i, __m128i), U256>((resultlo, resulthi));
    }
    unimplemented!()
}

#[allow(non_snake_case)]
const fn _MM_SHUFFLE(z: i32, y: i32, x: i32, w: i32) -> i32 {
    (z << 6) | (y << 4) | (x << 2) | w
}

#[allow(unreachable_code)]
unsafe fn shl_u256(count: U256, value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let one = _mm256_set_epi64x(0, 0, 0, 1);
        let sixty_four = _mm_set_epi64x(0, 64);
        let max_u8 = _mm256_sub_epi8(_mm256_setzero_si256(), one);
        let max_u64 = _mm256_sub_epi64(_mm256_setzero_si256(), one);
        //
        let count = std::mem::transmute::<U256, __m256i>(count);
        let value = std::mem::transmute::<U256, __m256i>(value);
        let hi248 = _mm256_andnot_si256(max_u8, count);
        let hiisz = broadcast_avx2(is_zero_u256(hi248.as_u256()));
        let mut temp = value;
        let mut current = _mm256_castsi256_si128(count);
        let mut i = 0;
        while i < 4 {
            let slcount = _mm_min_epu8(sixty_four, current);
            let srcount = _mm_subs_epu8(sixty_four, slcount);
            let sltemp = _mm256_sll_epi64(temp, slcount);
            let srtemp = _mm256_srl_epi64(temp, srcount);
            let carry = _mm256_permute4x64_epi64(srtemp, _MM_SHUFFLE(2, 1, 0, 3));
            temp = _mm256_or_si256(sltemp, _mm256_andnot_si256(max_u64, carry));
            current = _mm_subs_epu8(current, slcount);
            i += 1;
        }
        let result = _mm256_and_si256(temp, hiisz);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let zero = _mm_setzero_si128();
        let one = _mm_set_epi64x(0, 1);
        let sixty_four = _mm_set_epi64x(0, 64);
        let max_u8 = _mm_sub_epi8(zero, one);
        //
        let count = std::mem::transmute::<U256, (__m128i, __m128i)>(count);
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let hi248 = (_mm_andnot_si128(max_u8, count.0), count.1);
        let hi248 = std::mem::transmute::<(__m128i, __m128i), U256>(hi248);
        let hiisz = broadcast_sse2(is_zero_u256(hi248));
        let mut temp = value;
        let mut current = count.0;
        let mut i = 0;
        while i < 4 {
            let slcount = _mm_min_epu8(sixty_four, current);
            let srcount = _mm_subs_epu8(sixty_four, slcount);
            let sltemplo = _mm_sll_epi64(temp.0, slcount);
            let sltemphi = _mm_sll_epi64(temp.1, slcount);
            let srtemplo = _mm_srl_epi64(temp.0, srcount);
            let srtemphi = _mm_srl_epi64(temp.1, srcount);
            let carrylo = _mm_bslli_si128(srtemplo, 8);
            let carryhi = _mm_unpacklo_epi64(_mm_bsrli_si128(srtemplo, 8), srtemphi);
            let templo = _mm_or_si128(sltemplo, carrylo);
            let temphi = _mm_or_si128(sltemphi, carryhi);
            temp = (templo, temphi);
            current = _mm_subs_epu8(current, slcount);
            i += 1;
        }
        let result = (_mm_and_si128(hiisz, temp.0), _mm_and_si128(hiisz, temp.1));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    unimplemented!()
}

fn overflowing_add_u256(a: U256, b: U256) -> (U256, bool) {
    let t0 = (a.0[0] as u128) + (b.0[0] as u128);
    let c0 = t0 >> 64;
    let t1 = (a.0[1] as u128) + (b.0[1] as u128) + c0;
    let c1 = t1 >> 64;
    let t2 = (a.0[2] as u128) + (b.0[2] as u128) + c1;
    let c2 = t2 >> 64;
    let t3 = (a.0[3] as u128) + (b.0[3] as u128) + c2;
    let c3 = t3 >> 64;
    (U256([t0 as u64, t1 as u64, t2 as u64, t3 as u64]), c3 != 0)
}

fn add_u256(a: U256, b: U256) -> U256 {
    let (value, _) = overflowing_add_u256(a, b);
    value
}

fn mul_u64(a: u64, b: u64) -> u128 {
    /*
    #[cfg(target_feature = "bmi2")] {
        let lo: u64;
        let hi: u64;
        unsafe {
            asm!("mulxq $2, $1, $0"
                 : "=r"(hi), "=r"(lo)
                 : "r"(a), "{rdx}"(b)
                 );
        }
        return (lo as u128) | ((hi as u128) << 64);
    }
    */
    (a as u128) * (b as u128)
}

fn mul_diag(num_limbs: usize, i: usize, a: &[u64], b: u64, r: &mut [u64], c: &mut [u64]) {
    let mut carry: u64 = 0;
    for j in 0..num_limbs {
        let temp = mul_u64(a[j], b);
        if j == 0 {
            c[i] = temp as u64;
            carry = (temp >> 64) as u64;
        }
        else {
            let temp2 = temp + (carry as u128);
            if j == (num_limbs - 1) {
                r[j-1] = temp2 as u64;
                r[j-0] = (temp2 >> 64) as u64;
            }
            else {
                r[j-1] = temp2 as u64;
                carry = (temp2 >> 64) as u64;
            }
        }
    }
}

fn mul_diagc(num_limbs: usize, i: usize, a: &[u64], b: u64, r: &mut [u64], rp: &mut [u64], c: &mut [u64]) {
    let mut carry: u64 = 0;
    for j in 0..num_limbs {
        let temp = mul_u64(a[j], b) + (r[j] as u128);
        if j == 0 {
            c[i] = temp as u64;
            carry = (temp >> 64) as u64;
        }
        else {
            let temp2 = temp + (carry as u128);
            if j == (num_limbs - 1) {
                rp[j-1] = temp2 as u64;
                rp[j-0] = (temp2 >> 64) as u64;
            }
            else {
                rp[j-1] = temp2 as u64;
                carry = (temp2 >> 64) as u64;
            }
        }
    }
}

fn mul_limbs(num_limbs: usize, a: &[u64], b: &[u64], c: &mut [u64]) {
    assert!(num_limbs <= 4);
    let mut r: [u64; 8] = unsafe { std::mem::uninitialized() };
    let mut rp: [u64; 8] = unsafe { std::mem::uninitialized() };
    //
    mul_diag(num_limbs, 0, a, b[0], &mut r, c);
    for i in 1..num_limbs {
        mul_diagc(num_limbs, i, a, b[i], &mut r, &mut rp, c);
        for j in 0..num_limbs {
            r[j] = rp[j];
        }
    }
    for i in 0..num_limbs {
        c[num_limbs+i] = rp[i];
    }
}

fn mul_u256(a: U256, b: U256) -> U256 {
    let mut c: [u64; 8] = unsafe { std::mem::uninitialized() };
    mul_limbs(4, &a.0, &b.0, &mut c);
    U256([c[0], c[1], c[2], c[3]])
}

fn overflowing_sub_u256(a: U256, b: U256) -> (U256, bool) {
    let alo = ((a.0[1] as u128) << 64) | (a.0[0] as u128);
    let blo = ((b.0[1] as u128) << 64) | (b.0[0] as u128);
    let ahi = ((a.0[3] as u128) << 64) | (a.0[2] as u128);
    let bhi = ((b.0[3] as u128) << 64) | (b.0[2] as u128);
    let (lo, borrowlo) = alo.overflowing_sub(blo);
    let hi = ahi.wrapping_sub(bhi).wrapping_sub(borrowlo as u128);
    let borrow = (ahi < bhi) | ((ahi == bhi) & borrowlo);
    (U256([lo as u64, (lo >> 64) as u64, hi as u64, (hi >> 64) as u64]), borrow)
}

fn sub_u256(a: U256, b: U256) -> U256 {
    let (value, _) = overflowing_sub_u256(a, b);
    value
}

fn gt_u256(a: U256, b: U256) -> bool {
    let alo = ((a.0[1] as u128) << 64) | (a.0[0] as u128);
    let blo = ((b.0[1] as u128) << 64) | (b.0[0] as u128);
    let ahi = ((a.0[3] as u128) << 64) | (a.0[2] as u128);
    let bhi = ((b.0[3] as u128) << 64) | (b.0[2] as u128);
    (ahi > bhi) | ((ahi == bhi) & (alo > blo))
}

struct VmStackSlots([U256; VmStack::LEN]);

struct VmStack {
    start: *const U256,
    sp: *mut U256,
}

impl VmStack {
    pub const LEN: usize = 1024;

    pub unsafe fn new(slots: &mut VmStackSlots) -> VmStack {
        VmStack {
            start: slots.0.as_ptr(),
            // sp is always pointing at the top of the stack
            sp: slots.0.as_mut_ptr().offset(-1),
        }
    }

    pub unsafe fn push(&mut self, value: U256) {
        self.sp = self.sp.offset(1);
        store_u256(self.sp, value, 0);
    }

    pub unsafe fn pop(&mut self) -> U256 {
        let temp = self.peek();
        self.sp = self.sp.offset(-1);
        temp
    }

    pub unsafe fn peek(&self) -> U256 {
        self.peekn(0)
    }

    pub unsafe fn peek1(&self) -> U256 {
        self.peekn(1)
    }

    pub unsafe fn peekn(&self, index: usize) -> U256 {
        load_u256(self.sp, -(index as isize))
    }

    pub unsafe fn set(&self, index: usize, value: U256) -> U256 {
        let offset = -(index as isize);
        let temp = load_u256(self.sp, offset);
        store_u256(self.sp, value, offset);
        temp
    }

    pub unsafe fn size(&self) -> usize {
        const WORD_SIZE: usize = std::mem::size_of::<U256>();
        usize::wrapping_sub(self.sp.offset(1) as _, self.start as _) / WORD_SIZE
    }
}

struct VmMemory {
    ptr: *mut u8,
    len: usize
}

impl VmMemory {
    fn new() -> VmMemory {
        VmMemory {
            ptr: std::ptr::null_mut(),
            len: 0
        }
    }

    fn init(&mut self, gas: u64) {
        // TODO: compute worst-case capacity base on gas limit
        let max_len = 536870912; // 512M
        let capacity = max_len * 32;
        let layout = std::alloc::Layout::from_size_align(capacity, 32);
        match layout {
            Ok(layout) => {
                self.ptr = unsafe { std::alloc::alloc(layout) };
            },
            Err(e) => panic!(e)
        }
    }

    fn size(&self) -> usize {
        self.len * std::mem::size_of::<U256>()
    }

    fn extend(&mut self, offset: usize, size: usize) {
        let len = self.len;
        let new_len = (offset + size + 31) / 32;
        self.len = if new_len > len { new_len } else { len };
        self.len = if size == 0 { len } else { self.len };
    }

    unsafe fn read(&mut self, offset: usize) -> U256 {
        self.extend(offset, 32);
        let src = self.ptr.offset(offset as isize);
        let result = bswap_u256(loadu_u256(src as *const U256, 0));
        return result;
    }

    unsafe fn write(&mut self, offset: usize, value: U256) {
        self.extend(offset, 32);
        let dest = self.ptr.offset(offset as isize);
        storeu_u256(dest as *mut U256, bswap_u256(value), 0);
    }

    unsafe fn write_byte(&mut self, offset: usize, value: u8) {
        self.extend(offset, 1);
        let dest = self.ptr.offset(offset as isize);
        *dest = value;
    }

    unsafe fn slice(&self, offset: isize, size: usize) -> &[u8] {
        std::slice::from_raw_parts(self.ptr.offset(offset), size)
    }
}

macro_rules! comment {
   ($lit:literal) => (
        #[cfg(feature = "asm-comment")]
        {
            asm!(concat!("# ", $lit));
        }
    )
}

macro_rules! check_exception_at {
    ($addr:expr, $rom:ident, $stack:ident, $error:ident) => {
        let bb_info = $rom.get_bb_info($addr);
        let stack_min_size = bb_info.stack_min_size as usize;
        let stack_rel_max_size = bb_info.stack_rel_max_size as usize;
        const WORD_SIZE: usize = std::mem::size_of::<U256>();
        let stack_size = usize::wrapping_sub($stack.sp.offset(1) as _, $stack.start as _) / WORD_SIZE;
        let underflow = stack_size < stack_min_size;
        let overflow = (stack_size + stack_rel_max_size) > VmStack::LEN;
        if !(underflow | overflow) {
            continue;
        }
        if underflow {
            $error = VmError::StackUnderflow;
        } else if overflow {
            $error = VmError::StackOverflow;
        }
    }
}

#[derive(Debug)]
pub struct ReturnData {
    offset: usize,
    size: usize,
    gas: u64
}

impl ReturnData {
    pub fn new(offset: usize, size: usize, gas: u64) -> Self {
        ReturnData {
            offset: offset,
            size: size,
            gas: gas
        }
    }
}

fn lldb_hook_single_step(pc: usize, stsize: usize) {}
fn lldb_hook_stop(pc: usize, stsize: usize) {}

macro_rules! lldb_hook {
    ($code:expr, $gas:expr, $rom:ident, $stack:ident, $hook:ident) => {
        #[cfg(debug_assertions)]
        {
            let stack_start = $stack.start;
            const U8_SIZE: usize = std::mem::size_of::<u8>();
            let pc = usize::wrapping_sub($code as _, $rom.data.as_ptr() as _) / U8_SIZE;
            let stsize = $stack.size();
            $hook(pc, stsize);
        }
    }
}

unsafe fn run_evm(bytecode: &[u8], rom: &VmRom, memory: &mut VmMemory) -> ReturnData {
    // TODO: use MaybeUninit
    let mut slots: VmStackSlots = std::mem::uninitialized();
    let mut stack: VmStack = VmStack::new(&mut slots);
    let mut code: *const u8 = rom.code();
    let mut gas: u64 = 0;
    let mut error: VmError = VmError::None;
    let mut entered = false;
    while !entered {
        entered = true;
        check_exception_at!(0, rom, stack, error);
        panic!("{:?}", error);
    }
    loop {
        let opcode = std::mem::transmute::<u8, Opcode>(*code);
        lldb_hook!(code, gas, rom, stack, lldb_hook_single_step);
        //println!("{:?}", opcode);
        match opcode {
            STOP => {
                lldb_hook!(code, gas, rom, stack, lldb_hook_stop);
                break;
            },
            ADD => {
                comment!("opADD");
                let a = stack.pop();
                let b = stack.pop();
                let result = add_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            MUL => {
                comment!("opMUL");
                let a = stack.pop();
                let b = stack.pop();
                let result = mul_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            SUB => {
                comment!("opSUB");
                let a = stack.pop();
                let b = stack.pop();
                let result = sub_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            SIGNEXTEND => {
                comment!("opSIGNEXTEND");
                let offset = *(stack.sp as *const u32) % 32;
                let offset = offset as isize;
                let value = *((stack.sp.offset(-1) as *const u8).offset(offset));
                let a = stack.pop();
                let b = stack.pop();
                let result = signextend_u256(a, b, value as i64);
                stack.push(result);
                //
                code = code.offset(1);
            }
            GT => {
                comment!("opGT");
                let a = stack.pop();
                let b = stack.pop();
                let result = U256::from_u64(gt_u256(a, b) as u64);
                stack.push(result);
                //
                code = code.offset(1);
            }
            EQ => {
                comment!("opEQ");
                let a = stack.pop();
                let b = stack.pop();
                let result = eq_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            ISZERO => {
                comment!("opISZERO");
                let a = stack.pop();
                let result = iszero_u256(a);
                stack.push(result);
                //
                code = code.offset(1);
            }
            AND => {
                comment!("opAND");
                let a = stack.pop();
                let b = stack.pop();
                let result = and_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            OR => {
                comment!("opOR");
                let a = stack.pop();
                let b = stack.pop();
                let result = or_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            XOR => {
                comment!("opXOR");
                let a = stack.pop();
                let b = stack.pop();
                let result = xor_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            NOT => {
                comment!("opNOT");
                let a = stack.pop();
                let result = not_u256(a);
                stack.push(result);
                //
                code = code.offset(1);
            }
            BYTE => {
                comment!("opBYTE");
                let a = stack.peek();
                let lt32 = is_ltpow2_u256(a, 32);
                let offset = 31 - (a.0[0] % 32);
                let offset = offset as isize;
                let value = *((stack.sp.offset(-1) as *const u8).offset(offset));
                let value = value as u64;
                let result = U256::from_u64((lt32 as u64) * value);
                stack.pop();
                stack.pop();
                stack.push(result);
                //
                code = code.offset(1);
            }
            SHL => {
                comment!("opSHL");
                let a = stack.pop();
                let b = stack.pop();
                let result = shl_u256(a, b);
                stack.push(result);
                //
                code = code.offset(1);
            }
            POP => {
                comment!("opPOP");
                stack.pop();
                //
                code = code.offset(1);
            }
            MLOAD => {
                comment!("opMLOAD");
                let offset = stack.pop().low_u64();
                let result = memory.read(offset as usize);
                stack.push(result);
                //
                code = code.offset(1);
            },
            MSTORE => {
                comment!("opMSTORE");
                let offset = stack.pop().low_u64();
                let value = stack.pop();
                memory.write(offset as usize, value);
                //
                code = code.offset(1);
            },
            MSTORE8 => {
                comment!("opMSTORE8");
                let offset = stack.pop().low_u64();
                let value = stack.pop().low_u64();
                memory.write_byte(offset as usize, value as u8);
                //
                code = code.offset(1);
            },
            JUMP => {
                comment!("opJUMP");
                let addr = stack.pop();
                let in_bounds = is_ltpow2_u256(addr, VmRom::MAX_CODESIZE);
                let low = addr.low_u64();
                if in_bounds & rom.is_jumpdest(low) {
                    code = rom.code().offset(low as isize + 1);
                    check_exception_at!(low, rom, stack, error);
                    break;
                }
                else {
                    error = VmError::InvalidJumpDest;
                    break;
                }
            }
            JUMPI => {
                comment!("opJUMPI");
                let addr = stack.pop();
                let cond = stack.pop();
                if is_zero_u256(cond) {
                    code = code.offset(1);
                    const U8_SIZE: usize = std::mem::size_of::<u8>();
                    let pc = usize::wrapping_sub(code as _, rom.data.as_ptr() as _) / U8_SIZE;
                    check_exception_at!(pc as u64, rom, stack, error);
                    break;
                }
                else {
                    let in_bounds = is_ltpow2_u256(addr, VmRom::MAX_CODESIZE);
                    let low = addr.low_u64();
                    if in_bounds & rom.is_jumpdest(low) {
                        code = rom.code().offset(low as isize + 1);
                        check_exception_at!(low, rom, stack, error);
                        break;
                    }
                    else {
                        error = VmError::InvalidJumpDest;
                        break;
                    }
                }
            }
            PC => {
                comment!("opPC");
                let result = isize::wrapping_sub(code as _, rom.code() as _) - 1;
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                code = code.offset(1);
            }
            MSIZE => {
                comment!("opMSIZE");
                let result = U256::from_u64(memory.size() as u64);
                stack.push(result);
                //
                code = code.offset(1);
            }
            GAS => {
                comment!("opGAS");
                let result = U256::from_u64(0);
                stack.push(result);
                //
                code = code.offset(1);
            }
            JUMPDEST => {
                comment!("opJUMPDEST");
                //
                code = code.offset(1);
            }
            PUSH1 => {
                comment!("opPUSH1");
                code = code.offset(1);
                let result = *(code as *const u8);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                code = code.offset(1);
            }
            PUSH2 => {
                comment!("opPUSH2");
                code = code.offset(1);
                let result = *(code as *const u16);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                code = code.offset(2);
            }
            PUSH4 => {
                comment!("opPUSH4");
                code = code.offset(1);
                let result = *(code as *const u32);
                let result = U256::from_u64(result as u64);
                stack.push(result);
                //
                code = code.offset(4);
            }
            PUSH3 | PUSH5 | PUSH6 | PUSH7 | PUSH8 | PUSH9 | PUSH10 | PUSH11 |
            PUSH12 | PUSH13 | PUSH14 | PUSH15 | PUSH16 => {
                comment!("opPUSH16");
                code = code.offset(1);
                let num_bytes = (opcode.push_index() as i32) + 1;
                let result = load16_u256(code as *const U256, num_bytes);
                stack.push(result);
                //
                code = code.offset(num_bytes as isize);
            }
            PUSH17 | PUSH18 | PUSH19 | PUSH20 | PUSH21 | PUSH22 | PUSH23 |
            PUSH24 | PUSH25 | PUSH26 | PUSH27 | PUSH28 | PUSH29 | PUSH30 |
            PUSH31 | PUSH32 => {
                comment!("opPUSH32");
                code = code.offset(1);
                let num_bytes = (opcode.push_index() as i32) + 1;
                let result = load32_u256(code as *const U256, num_bytes);
                stack.push(result);
                //
                code = code.offset(num_bytes as isize);
            }
            DUP1 => {
                comment!("opDUP1");
                let result = stack.peek();
                stack.push(result);
                //
                code = code.offset(1);
            }
            DUP2 => {
                comment!("opDUP2");
                let result = stack.peek1();
                stack.push(result);
                //
                code = code.offset(1);
            }
            DUP3 | DUP4 | DUP5 | DUP6 | DUP7 | DUP8 | DUP9 | DUP10 | DUP11 |
            DUP12 | DUP13 | DUP14 | DUP15 | DUP16 => {
                comment!("opDUPn");
                let index = opcode.dup_index();
                let result = stack.peekn(index);
                stack.push(result);
                //
                code = code.offset(1);
            }
            SWAP1 => {
                comment!("opSWAP1");
                let a = stack.pop();
                let b = stack.pop();
                stack.push(a);
                stack.push(b);
                //
                code = code.offset(1);
            }
            SWAP2 => {
                comment!("opSWAP2");
                let value = stack.peek();
                let prev = stack.set(2, value);
                stack.pop();
                stack.push(prev);
                //
                code = code.offset(1);
            }
            SWAP3 | SWAP4 | SWAP5 | SWAP6 | SWAP7 | SWAP8 | SWAP9 | SWAP10 |
            SWAP11 | SWAP12 | SWAP13 | SWAP14 | SWAP15 | SWAP16 => {
                comment!("opSWAPn");
                let value = stack.peek();
                let index = opcode.swap_index();
                let prev = stack.set(index, value);
                stack.pop();
                stack.push(prev);
                //
                code = code.offset(1);
            }
            RETURN => {
                lldb_hook!(code, gas, rom, stack, lldb_hook_stop);
                comment!("opRETURN");
                let offset = stack.pop();
                let size = stack.pop();
                let offset = offset.low_u64() as usize;
                let size = size.low_u64() as usize;
                return ReturnData::new(offset, size, 0)
            }
            INVALID => {
                error = VmError::InvalidInstruction;
                break;
            }
        }
    }
    if let VmError::None = error {
        return ReturnData::new(0, 0, 0);
    }
    panic!("{:?}", error);
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
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

#[derive(Debug)]
struct BbInfo {
    stack_min_size: u16,
    stack_rel_max_size: u16,
    gas: u32,
}
impl BbInfo {
    fn new(stack_min_size: u16, stack_max_size: u16) -> BbInfo {
        let stack_rel_max_size = if stack_max_size > stack_min_size {
            stack_max_size - stack_min_size
        }
        else {
            0
        };
        BbInfo {
            stack_min_size,
            stack_rel_max_size: stack_rel_max_size,
            gas: 0,
        }
    }
}

struct VmRom {
    data: [u8; VmRom::SIZE]
}

impl VmRom {
    /// EIP-170 states a max contract code size of 2**14 + 2**13, we round it
    /// to the next power of two.
    const MAX_CODESIZE: usize = 32768;

    const JUMPDESTS_SIZE: usize = VmRom::MAX_CODESIZE / 8;

    const BB_INFOS_SIZE: usize = VmRom::MAX_CODESIZE * std::mem::size_of::<BbInfo>();

    const SIZE: usize = VmRom::MAX_CODESIZE + VmRom::JUMPDESTS_SIZE + VmRom::BB_INFOS_SIZE;

    const BB_INFOS_OFFSET: usize = VmRom::MAX_CODESIZE + VmRom::JUMPDESTS_SIZE;

    fn new() -> VmRom {
        VmRom { data: [0; VmRom::SIZE] }
    }

    fn code(&self) -> *const u8 {
        self.data.as_ptr()
    }

    fn is_jumpdest(&self, addr: u64) -> bool {
        let jump_dests = unsafe {
            self.data.as_ptr().offset(VmRom::MAX_CODESIZE as isize) as *mut u64
        };
        let offset = (addr % (VmRom::MAX_CODESIZE as u64)) as isize;
        let bits = unsafe { *jump_dests.offset(offset / 64) };
        let mask = 1u64 << (offset % 64);
        (bits & mask) != 0
    }

    fn get_bb_info(&self, addr: u64) -> &BbInfo {
        unsafe {
            let offset = VmRom::BB_INFOS_OFFSET as isize;
            let bb_infos = self.data.as_ptr().offset(offset) as *mut BbInfo;
            &*bb_infos.offset(addr as isize)
        }
    }

    fn swap_bytes(input: &[u8], swapped: &mut[u8]) {
        for i in 0..input.len() {
            swapped[input.len()-1-i] = input[i];
        }
    }

    fn write_bb_infos(&mut self, bytecode: &[u8]) {
        use std::cmp::max;
        #[derive(Debug)]
        struct BlockInfo {
            addr: u32,
            stack_min_size: u16,
            stack_max_size: u16,
            stack_end_size: u16,
            is_basic_block: bool,
        }
        impl BlockInfo {
            fn basic(addr: u32, stack_min_size: u16, stack_max_size: u16, stack_end_size: u16) -> BlockInfo {
                BlockInfo {
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_end_size,
                    is_basic_block: true,
                }
            }
            fn partial(addr: u32, stack_min_size: u16, stack_max_size: u16, stack_end_size: u16) -> BlockInfo {
                BlockInfo {
                    addr,
                    stack_min_size,
                    stack_max_size,
                    stack_end_size,
                    is_basic_block: false,
                }
            }
        }
        const DELTA_ALPHAS: [(u16, u16); 256] = [(0, 0), (2, 1), (2, 1), (2, 1), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (2, 1), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (2, 1), (0, 0), (0, 0), (2, 1), (1, 1), (2, 1), (2, 1), (2, 1), (1, 1), (2, 1), (2, 1), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (1, 0), (1, 1), (2, 0), (2, 0), (0, 0), (0, 0), (1, 0), (2, 0), (0, 1), (0, 1), (0, 1), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (7, 8), (8, 9), (9, 10), (10, 11), (11, 12), (12, 13), (13, 14), (14, 15), (15, 16), (16, 17), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7), (8, 8), (9, 9), (10, 10), (11, 11), (12, 12), (13, 13), (14, 14), (15, 15), (16, 16), (17, 17), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (2, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)];
        let mut addr: u32 = 0;
        let mut stack_size: u16 = 0;
        let mut stack_min_size: u16 = 0;
        let mut stack_max_size: u16 = 0;
        let mut block_infos: Vec<BlockInfo> = Vec::with_capacity(1024);
        // forward pass over the bytecode
        let mut i: usize = 0;
        while i < bytecode.len() {
            let code = bytecode[i];
            let opcode = unsafe {
                std::mem::transmute::<u8, EvmOpcode>(code)
            };
            let (delta, alpha) = DELTA_ALPHAS[code as usize];
            // new_stack_size is (stack_size + needed + alpha) - delta
            // and represents the new stack size after the opcode has been
            // dispatched
            let (new_stack_size, needed) = if delta > stack_size {
                    (alpha, (delta - stack_size))
                } else {
                    // case stack_size >= delta
                    ((stack_size - delta).saturating_add(alpha), 0)
                };
            stack_size = new_stack_size;
            stack_min_size = stack_min_size.saturating_add(needed);
            stack_max_size = max(stack_max_size, new_stack_size);
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                i += 1 + num_bytes;
            }
            else {
                i += 1;
            }
            if opcode.is_terminator() || i >= bytecode.len() {
                block_infos.push(BlockInfo::basic(
                    addr, stack_min_size, stack_max_size, stack_size)
                );
                addr = i as u32;
                stack_size = 0;
                stack_min_size = 0;
                stack_max_size = 0;
            }
            else {
                let code = bytecode[i];
                let opcode = unsafe {
                    std::mem::transmute::<u8, EvmOpcode>(code)
                };
                if opcode == EvmOpcode::JUMPDEST {
                    block_infos.push(BlockInfo::partial(
                        addr, stack_min_size, stack_max_size, stack_size)
                    );
                    addr = i as u32;
                    stack_size = 0;
                    stack_min_size = 0;
                    stack_max_size = 0;
                }
            }
        }
        // backward pass, write BB infos to rom
        let bb_infos = unsafe {
            let offset = VmRom::BB_INFOS_OFFSET as isize;
            self.data.as_ptr().offset(offset) as *mut BbInfo
        };
        for info in block_infos.iter().rev() {
            if info.is_basic_block {
                stack_min_size = info.stack_min_size;
                stack_max_size = info.stack_max_size;
            }
            else {
                let (more, needed) = if stack_min_size > info.stack_end_size {
                    (0, (stack_min_size - info.stack_end_size))
                } else {
                    // case info.stack_end_size >= stack_min_size
                    (info.stack_end_size - stack_min_size, 0)
                };
                stack_min_size = info.stack_min_size.saturating_add(needed);
                stack_max_size = max(
                    info.stack_max_size.saturating_add(needed),
                    stack_max_size.saturating_add(more)
                );
            }
            unsafe {
                let bb_info = BbInfo::new(stack_min_size, stack_max_size);
                *bb_infos.offset(info.addr as isize) = bb_info;
            }
        }
    }

    fn init(&mut self, bytecode: &[u8]) {
        // erase rom
        for b in &mut self.data[..] {
            *b = 0;
        }
        if bytecode.len() > VmRom::MAX_CODESIZE {
            panic!("bytecode is too big ({:?} bytes)", bytecode.len());
        }
        // copy bytecode
        #[cfg(target_endian = "little")]
        {
            // reverse `PUSHN` opcode bytes
            let mut i: usize = 0;
            while i < bytecode.len() {
                let code = bytecode[i];
                let opcode = unsafe {
                    std::mem::transmute::<u8, EvmOpcode>(code)
                };
                self.data[i] = opcode.to_internal() as u8;
                if opcode.is_push() {
                    let num_bytes = opcode.push_index() + 1;
                    let start = i + 1;
                    let end = start + num_bytes;
                    let dest = &mut self.data[start..end];
                    VmRom::swap_bytes(&bytecode[start..end], dest);
                    i += 1 + num_bytes;
                }
                else {
                    i += 1;
                }
            }
        }
        #[cfg(target_endian = "big")]
        {
            unimplemented!();
        }
        // write valid jump destinations
        let jump_dests_offset = VmRom::MAX_CODESIZE as isize;
        let jump_dests = unsafe {
            self.data.as_mut_ptr().offset(jump_dests_offset) as *mut u64
        };
        let mut bits: u64 = 0;
        let mut i: usize = 0;
        while i < bytecode.len() {
            // save i for later in j
            let j = i;
            let code = bytecode[i];
            let opcode = unsafe {
                std::mem::transmute::<u8, EvmOpcode>(code)
            };
            if opcode.is_push() {
                let num_bytes = opcode.push_index() + 1;
                i += 1 + num_bytes;
            }
            else {
                if opcode == EvmOpcode::JUMPDEST {
                    bits |= 1u64 << (i % 64);
                }
                i += 1;
            }
            let do_write = (j % 64) > (i % 64);
            if do_write {
                let offset = (j / 64) as isize;
                unsafe {
                    *jump_dests.offset(offset) = bits
                }
                bits = 0;
            }
        }
        let offset = (i / 64) as isize;
        unsafe {
            *jump_dests.offset(offset) = bits
        }
        //
        self.write_bb_infos(bytecode);
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut temp = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(&mut temp, "{:02x}", b);
    }
    temp
}

#[allow(unreachable_code)]
fn print_config() {
    #[cfg(debug_assertions)]
    {
        println!("mode: debug");
    }
    #[cfg(not(debug_assertions))]
    {
        println!("mode: release");
    }
    #[cfg(target_feature = "avx2")]
    {
        println!("path: AVX2");
        return;
    }
    #[cfg(target_feature = "ssse3")]
    {
        println!("path: SSSE3");
        return;
    }
}

const VM_DEFAULT_GAS: u64 = 20_000_000_000_000;

struct Bytecode<'a> {
    data: &'a [u8],
    addr: usize
}

impl<'a> Bytecode<'a> {
    fn new(bytes: &'a [u8]) -> Bytecode<'a> {
        Bytecode { data: bytes, addr: 0 }
    }
}

struct IncompletePushError {
    addr: usize
}

impl fmt::Display for IncompletePushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incomplete push instruction at 0x{:04x}", self.addr)
    }
}

impl<'a> Iterator for Bytecode<'a> {
    type Item = Result<EvmInstruction<'a>, IncompletePushError>;
    fn next(&mut self) -> Option<Result<EvmInstruction<'a>, IncompletePushError>> {
        if self.addr < self.data.len() {
            let value = self.data[self.addr];
            match EvmOpcode::try_from(value) {
                Ok(opcode) => {
                    if opcode.is_push() {
                        let num_bytes = opcode.push_index() + 1;
                        let start = self.addr + 1;
                        let end = start + num_bytes;
                        if (end-1) < self.data.len() {
                            let temp = EvmInstruction::MultiByte {
                                addr: self.addr,
                                opcode: opcode,
                                bytes: &self.data[start..end]
                            };
                            self.addr += 1 + num_bytes;
                            Some(Ok(temp))
                        } else {
                            Some(Err(IncompletePushError { addr: self.addr }))
                        }
                    }
                    else {
                        let temp = EvmInstruction::SingleByte {
                            addr: self.addr,
                            opcode: opcode
                        };
                        self.addr += 1;
                        Some(Ok(temp))
                    }
                },
                Err(_) => {
                    let temp = EvmInstruction::SingleByte {
                        addr: self.addr,
                        opcode: EvmOpcode::INVALID
                    };
                    self.addr += 1;
                    Some(Ok(temp))
                }
            }
        } else {
            None
        }
    }
}

fn disasm(input: &str) {
    let temp = decode_hex(input);
    match temp {
        Ok(bytes) => {
            let result: Result<Vec<EvmInstruction>, _> = Bytecode::new(&bytes).collect();
            match result {
                Ok(x) => {
                    let asm = x
                        .iter()
                        .map(|i| match i {
                            EvmInstruction::SingleByte { addr, opcode } => {
                                format!("{:04x}:    {}", addr, opcode)
                            },
                            EvmInstruction::MultiByte { addr, opcode, bytes } => {
                                let imm = encode_hex(bytes);
                                format!("{:04x}:    {} 0x{}", addr, opcode, imm)
                            },
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    println!("{}", asm);
                },
                Err(e) => println!("{}", e)
            }
        }
        Err(e) => println!("{:?}", e)
    }
}

fn evm(input: &str) {
    let temp = decode_hex(input);
    match temp {
        Ok(bytes) => {
            //println!("{} bytes", bytes.len());
            let mut rom = VmRom::new();
            rom.init(&bytes);
            let mut memory = VmMemory::new();
            memory.init(VM_DEFAULT_GAS);
            let slice = unsafe {
                let ret_data = run_evm(&bytes, &rom, &mut memory);
                memory.slice(ret_data.offset as isize, ret_data.size)
            };
            let mut buffer = String::with_capacity(512);
            for byte in slice {
                let _ = write!(buffer, "{:02x}", byte);
            }
            println!("0x{:}", buffer);
        },
        Err(e) => println!("{:?}", e)
    };
}

fn main() {
    let matches =
        App::new("Psyche")
            .subcommand(SubCommand::with_name("evm")
                .about("Run EVM bytecode")
                .arg(Arg::with_name("input")
                    .index(1)
                    .required(true)
                    .help("A valid hex string like 60ff60010100")))
            .subcommand(SubCommand::with_name("disasm")
                .about("Disassemble EVM bytecode")
                .arg(Arg::with_name("input")
                    .index(1)
                    .required(true)
                    .help("A valid hex string like 60ff60010100")))
            .get_matches();

    if let Some(matches) = matches.subcommand_matches("disasm") {
        let input = matches.value_of("input").unwrap();
        disasm(input);
        return;
    }
    if let Some(matches) = matches.subcommand_matches("evm") {
        let input = matches.value_of("input").unwrap();
        evm(input);
        return;
    }
}
