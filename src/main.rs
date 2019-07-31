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

#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod instructions;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use num_traits::FromPrimitive;
use std::env;
use std::{fmt::Write, num::ParseIntError};
use instructions::{Instruction};
use instructions::Instruction::*;

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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return false;
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
    return false;
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
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
    return U256([0u64; 4]);
}

const VM_STACK_SIZE: usize = 1024;
const MAX_CODESIZE: usize = 32768;

#[repr(align(32))]
struct VmStackSlots([U256; VM_STACK_SIZE]);

struct VmStack {
    sp: *mut U256,
    st0: U256,
    st1: U256
}

impl VmStack {
    pub unsafe fn new(slots: &mut VmStackSlots) -> VmStack {
        VmStack {
            // offset of 1 is needed to streamline memory accesses
            sp: slots.0.as_mut_ptr().offset(1),
            st0: U256([0u64; 4]),
            st1: U256([0u64; 4])
        }
    }

    pub unsafe fn push(&mut self, value: U256) {
        store_u256(self.sp, self.st1, -1);
        self.sp = self.sp.offset(1);
        self.st1 = self.st0;
        self.st0 = value;
    }

    pub unsafe fn pop(&mut self) -> U256 {
        let result = self.st0;
        self.sp = self.sp.offset(-1);
        self.st0 = self.st1;
        self.st1 = load_u256(self.sp, -1);
        return result;
    }

    pub unsafe fn peek(&self) -> U256 {
        return self.st0;
    }

    pub unsafe fn peek1(&self) -> U256 {
        return self.st1;
    }

    pub unsafe fn peekn(&self, position: usize) -> U256 {
        return load_u256(self.sp, -(position as isize));
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

#[inline(never)]
unsafe fn run_evm(bytecode: &[Instruction]) -> U256 {
    // TODO: use MaybeUninit
    let mut slots: VmStackSlots = std::mem::uninitialized();
    let mut stack: VmStack = VmStack::new(&mut slots);
    let mut code: *const Instruction = bytecode.as_ptr();
    loop {
        let opcode: Instruction = *code;
        //println!("{:?}", opcode);
        match opcode {
            STOP => {
                break;
            },
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
                let result = U256 { 0: [(lt32 as u64) * value, 0, 0, 0] };
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
            JUMP => {
                comment!("opJUMP");
                let addr = stack.pop();
                let in_bounds = is_ltpow2_u256(addr, MAX_CODESIZE);
                if in_bounds {
                    code = bytecode.as_ptr().offset(addr.low_u64() as isize);
                    continue;
                }
            }
            JUMPI => {
                comment!("opJUMPI");
                let addr = stack.pop();
                let cond = stack.pop();
                if is_zero_u256(cond) {
                    code = code.offset(1);
                }
                else {
                    let in_bounds = is_ltpow2_u256(addr, MAX_CODESIZE);
                    if in_bounds {
                        let offset = addr.low_u64();
                        code = bytecode.as_ptr().offset(offset as isize);
                        continue;
                    }
                }
            }
            PC => {
                comment!("opPC");
                let result = isize::wrapping_sub(code as _, bytecode.as_ptr() as _) - 1;
                let result = U256::from_u64(result as u64);
                stack.push(result);
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
                let num_bytes = opcode.push_bytes() as i32;
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
                let num_bytes = opcode.push_bytes() as i32;
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
            DUP3 | DUP4 => {
                comment!("opDUPn");
                let position = opcode.dup_position();
                let result = stack.peekn(position);
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
            INVALID => {
                break;
            }
        }
    }
    return stack.pop();
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

fn build_bytecode(bytes: Vec<u8>) -> Vec<Instruction> {
    let mut bytecode = vec!();
    bytecode.reserve(bytecode.len());
    let mut i = 0;
    while i < bytes.len() {
        match Instruction::from_u8(bytes[i]) {
            Some(instr) => {
                if instr.is_push() {
                    let mut buffer: [u8; 32] = [0; 32];
                    let num_bytes = instr.push_bytes() as usize;
                    let mut j = 0;
                    while j < num_bytes {
                        buffer[j] = bytes[i+num_bytes-j];
                        j += 1;
                    }
                    //
                    bytecode.push(instr);
                    j = 0;
                    while j < num_bytes {
                        let f: Instruction = unsafe {
                            std::mem::transmute::<u8, Instruction>(buffer[j])
                        };
                        bytecode.push(f);
                        j += 1;
                    }
                    i += 1 + num_bytes;
                }
                else {
                    bytecode.push(instr);
                    i += 1;
                }
            }
            None => bytecode.push(Instruction::INVALID)
        }
    }
    // Pad with 15 STOP instructions
    for _ in 0..15 {
        bytecode.push(Instruction::STOP)
    }
    bytecode
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

fn main() {
    print_config();
    if let Some(arg1) = env::args().nth(1) {
        let temp = decode_hex(&arg1);
        match temp {
            Ok(bytes) => {
                println!("{} bytes", bytes.len());
                let bytecode = build_bytecode(bytes);
                let result: U256 = unsafe {
                    run_evm(&bytecode)
                };
                println!("0x{:016x}{:016x}{:016x}{:016x}",
                    result.0[3],
                    result.0[2],
                    result.0[1],
                    result.0[0]);
            },
            Err(e) => println!("{:?}", e)
        };
    }
    else {
        println!("The first positional argument must be a hex string");
    }
}
