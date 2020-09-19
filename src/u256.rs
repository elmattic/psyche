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

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem::MaybeUninit;

#[repr(align(32))]
#[derive(Copy, Clone)]
pub struct U256(pub [u64; 4]);

#[repr(align(32))]
#[derive(Copy, Clone)]
pub struct U512(pub [u64; 8]);

impl U256 {
    pub fn default() -> U256 {
        U256 { 0: [0, 0, 0, 0] }
    }

    pub fn from_slice(value: &[u64]) -> U256 {
        U256 {
            0: [value[0], value[1], value[2], value[3]],
        }
    }

    pub fn from_dec_str(value: &str) -> Result<U256, uint::FromDecStrErr> {
        match ethereum_types::U256::from_dec_str(value) {
            Ok(temp) => {
                let mask = ethereum_types::U256::from(u64::max_value());
                let data: [u64; 4] = [
                    ((temp >> 0) & mask).as_u64(),
                    ((temp >> 64) & mask).as_u64(),
                    ((temp >> 128) & mask).as_u64(),
                    ((temp >> 192) & mask).as_u64(),
                ];
                Ok(U256::from_slice(&data))
            }
            Err(err) => Err(err),
        }
    }

    pub fn from_u64(value: u64) -> U256 {
        U256 {
            0: [value, 0, 0, 0],
        }
    }

    pub fn from_bool(value: bool) -> U256 {
        U256 {
            0: [value as u64, 0, 0, 0],
        }
    }

    pub fn broadcast_u64(value: u64) -> U256 {
        U256 {
            0: [value, value, value, value],
        }
    }

    pub fn low_u64(&self) -> u64 {
        self.0[0]
    }

    pub fn low_u128(&self) -> u128 {
        let lo = self.0[0] as u128;
        let hi = self.0[1] as u128;
        (hi << 64) | lo
    }

    pub fn high_u128(&self) -> u128 {
        let lo = self.0[2] as u128;
        let hi = self.0[3] as u128;
        (hi << 64) | lo
    }

    pub fn le_u64(&self) -> bool {
        (self.0[1] == 0) & (self.0[2] == 0) & (self.0[3] == 0)
    }
}

pub trait __m256iExt {
    unsafe fn as_u256(&self) -> U256;
}

impl __m256iExt for __m256i {
    unsafe fn as_u256(&self) -> U256 {
        return std::mem::transmute::<__m256i, U256>(*self);
    }
}

#[cfg(target_feature = "avx2")]
#[derive(Copy, Clone)]
#[repr(align(32))]
pub struct Word(pub __m256i);

#[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
#[derive(Copy, Clone)]
#[repr(align(32))]
pub struct Word(pub (__m128i, __m128i));

#[cfg(not(target_feature = "ssse3"))]
#[derive(Copy, Clone)]
#[repr(align(32))]
pub struct Word(pub U256);

impl Word {
    pub unsafe fn as_u256(&self) -> U256 {
        std::mem::transmute::<Word, U256>(*self)
    }

    pub unsafe fn from_slice(value: &[u64]) -> Word {
        #[cfg(target_feature = "avx2")]
        {
            return Word(_mm256_set_epi64x(
                value[3] as i64,
                value[2] as i64,
                value[1] as i64,
                value[0] as i64,
            ));
        }
        #[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
        {
            return Word((
                _mm_set_epi64x(value[1] as i64, value[0] as i64),
                _mm_set_epi64x(value[3] as i64, value[2] as i64),
            ));
        }
        #[cfg(not(target_feature = "ssse3"))]
        {
            Word(U256::from_slice(value))
        }
    }

    pub unsafe fn from_u64(value: u64) -> Word {
        #[cfg(target_feature = "avx2")]
        {
            return Word(_mm256_set_epi64x(0, 0, 0, value as i64));
        }
        #[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
        {
            return Word((_mm_set_epi64x(0, value as i64), _mm_setzero_si128()));
        }
        #[cfg(not(target_feature = "ssse3"))]
        {
            Word(U256::from_u64(value))
        }
    }
}

#[allow(unreachable_code)]
pub unsafe fn load_u256(src: *const U256, offset: isize) -> U256 {
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
    // generic target
    return *src.offset(offset);
}

#[allow(unreachable_code)]
pub unsafe fn loadu_u256(src: *const U256, offset: isize) -> U256 {
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
    // generic target
    return *src.offset(offset);
}

#[allow(unreachable_code)]
pub unsafe fn store_u256(dest: *mut U256, value: U256, offset: isize) {
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
    // generic target
    *dest.offset(offset) = value;
}

#[allow(unreachable_code)]
pub unsafe fn storeu_u256(dest: *mut U256, value: U256, offset: isize) {
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
    // generic target
    *dest.offset(offset) = value;
}

fn bitmask(num_bytes: i32) -> u64 {
    let f = -((num_bytes != 0) as i64) as u64;
    f & u64::max_value().wrapping_shr(64 - 8 * num_bytes as u32)
}

#[allow(unreachable_code)]
pub unsafe fn load16_u256(src: *const U256, num_bytes: i32) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane8_id = _mm256_set_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        );
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
    // generic target
    let src = src as *const u64;
    if num_bytes <= 8 {
        let mask: u64 = bitmask(num_bytes - 0);
        let temp0 = *src.offset(0) & mask;
        U256([temp0, 0, 0, 0])
    } else {
        let mask: u64 = bitmask(num_bytes - 8);
        let temp0 = *src.offset(0);
        let temp1 = *src.offset(1) & mask;
        U256([temp0, temp1, 0, 0])
    }
}

#[allow(unreachable_code)]
pub unsafe fn load32_u256(src: *const U256, num_bytes: i32) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane8_id = _mm256_set_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        );
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
        return std::mem::transmute::<(__m128i, __m128i), U256>((
            valuelo,
            _mm_and_si128(valuehi, mask),
        ));
    }
    // generic target
    let src = src as *const u64;
    if num_bytes <= 24 {
        let mask: u64 = bitmask(num_bytes - 16);
        let temp0 = *src.offset(0);
        let temp1 = *src.offset(1);
        let temp2 = *src.offset(2) & mask;
        U256([temp0, temp1, temp2, 0])
    } else {
        let mask: u64 = bitmask(num_bytes - 24);
        let temp0 = *src.offset(0);
        let temp1 = *src.offset(1);
        let temp2 = *src.offset(2);
        let temp3 = *src.offset(3) & mask;
        U256([temp0, temp1, temp2, temp3])
    }
}

#[allow(unreachable_code)]
pub unsafe fn bswap_u256(value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane8_id = _mm256_set_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        );
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
    // generic target
    U256([
        value.0[3].swap_bytes(),
        value.0[2].swap_bytes(),
        value.0[1].swap_bytes(),
        value.0[0].swap_bytes(),
    ])
}

#[allow(unreachable_code)]
pub unsafe fn is_zero_u256(value: U256) -> bool {
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
    // generic target
    (value.0[0] == 0) & (value.0[1] == 0) & (value.0[2] == 0) & (value.0[3] == 0)
}

#[allow(unreachable_code)]
pub unsafe fn is_ltpow2_u256(value: U256, pow2: usize) -> bool {
    debug_assert!(pow2.is_power_of_two());
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
    // generic target
    let mask = (pow2 as u64) - 1;
    let temp = U256([value.0[0] & !mask, value.0[1], value.0[2], value.0[3]]);
    let result = is_zero_u256(temp);
    return result;
}

#[cfg(target_feature = "avx2")]
unsafe fn broadcast_avx2(value: bool) -> __m256i {
    let mask = _mm_set_epi32(0, 0, 0, if value { -1 } else { 0 });
    return _mm256_broadcastd_epi32(mask);
}

#[cfg(target_feature = "sse2")]
unsafe fn broadcast_sse2(value: bool) -> __m128i {
    let mask = _mm_set_epi32(0, 0, 0, if value { -1 } else { 0 });
    return _mm_shuffle_epi32(mask, 0);
}

#[cfg(target_feature = "sse2")]
#[allow(unreachable_code)]
unsafe fn mm_blendv_epi8(a: __m128i, b: __m128i, mask: __m128i) -> __m128i {
    #[cfg(target_feature = "sse4.1")]
    {
        return _mm_blendv_epi8(a, b, mask);
    }
    return _mm_or_si128(_mm_and_si128(b, mask), _mm_andnot_si128(mask, a));
}

#[cfg(target_feature = "sse2")]
#[allow(unreachable_code)]
unsafe fn mm_cmpeq_epi64(a: __m128i, b: __m128i) -> __m128i {
    #[cfg(target_feature = "sse4.1")]
    {
        return _mm_cmpeq_epi64(a, b);
    }
    let mask = _mm_cmpeq_epi32(a, b);
    let smask = _mm_shuffle_epi32(mask, _MM_SHUFFLE(2, 3, 0, 1));
    return _mm_and_si128(mask, smask);
}

#[cfg(target_feature = "sse2")]
#[allow(unreachable_code)]
unsafe fn mm_select_si128(a: __m128i, b: __m128i, mask: __m128i) -> __m128i {
    #[cfg(target_feature = "sse4.1")]
    {
        return _mm_blendv_epi8(a, b, mask);
    }
    return _mm_or_si128(_mm_and_si128(b, mask), a);
}

fn blend_u64(a: u64, b: u64, mask: u64) -> u64 {
    //a ^ ((a ^ b) & mask)
    (b & mask) | (a & !mask)
}

fn bitmask_bool(value: bool) -> u64 {
    let f = value as i64;
    ((-f) as u64) & u64::max_value()
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.min(max).max(min)
}

#[allow(unreachable_code)]
pub unsafe fn signextend_u256(a: U256, b: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        const SWAP_LANE128: i32 = (1 << 0) + (0 << 4);
        let lane8_rev_id = _mm256_set_epi8(
            31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
            9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
        );
        //
        let _a = std::mem::transmute::<U256, __m256i>(a);
        let _b = std::mem::transmute::<U256, __m256i>(b);
        let alo = _mm256_castsi256_si128(_a);
        let alob = _mm256_broadcastb_epi8(alo);
        let bmask = _mm256_cmpeq_epi8(lane8_rev_id, alob);
        let bsel = _mm256_and_si256(bmask, _b);
        let isneg = _mm256_cmpgt_epi8(_mm256_setzero_si256(), bsel);
        let x = _mm256_shuffle_epi8(isneg, alob);
        let px = _mm256_permute2x128_si256(x, x, SWAP_LANE128);
        let signmask = _mm256_or_si256(x, px);
        let mask = _mm256_cmpgt_epi8(lane8_rev_id, alob);
        let temp = _mm256_blendv_epi8(_b, signmask, mask);
        let lt32 = broadcast_avx2(is_ltpow2_u256(a, 32));
        let result = _mm256_blendv_epi8(_b, temp, lt32);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let zero = _mm_setzero_si128();
        let lane8_rev_id_lo = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let lane8_rev_id_hi = _mm_set_epi8(
            31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16,
        );
        //
        let _a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let _b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let alob = _mm_shuffle_epi8(_a.0, zero);
        let bmasklo = _mm_cmpeq_epi8(lane8_rev_id_lo, alob);
        let bmaskhi = _mm_cmpeq_epi8(lane8_rev_id_hi, alob);
        let bsello = _mm_and_si128(bmasklo, _b.0);
        let bselhi = _mm_and_si128(bmaskhi, _b.1);
        let isneglo = _mm_cmpgt_epi8(zero, bsello);
        let isneghi = _mm_cmpgt_epi8(zero, bselhi);
        let xlo = _mm_shuffle_epi8(isneglo, alob);
        let xhi = _mm_shuffle_epi8(isneghi, alob);
        let signmask = _mm_or_si128(xlo, xhi);
        let masklo = _mm_cmpgt_epi8(lane8_rev_id_lo, alob);
        let maskhi = _mm_cmpgt_epi8(lane8_rev_id_hi, alob);
        let templo = mm_blendv_epi8(_b.0, signmask, masklo);
        let temphi = mm_blendv_epi8(_b.1, signmask, maskhi);
        let lt32 = broadcast_sse2(is_ltpow2_u256(a, 32));
        let resultlo = mm_blendv_epi8(_b.0, templo, lt32);
        let resulthi = mm_blendv_epi8(_b.1, temphi, lt32);
        return std::mem::transmute::<(__m128i, __m128i), U256>((resultlo, resulthi));
    }
    // generic target
    let _a = a.low_u64() & 31;
    let num_bytes = _a as i32 + 1;
    let mask0 = bitmask(clamp_i32(num_bytes - 0, 0, 8));
    let mask1 = bitmask(clamp_i32(num_bytes - 8, 0, 8));
    let mask2 = bitmask(clamp_i32(num_bytes - 16, 0, 8));
    let mask3 = bitmask(clamp_i32(num_bytes - 24, 0, 8));
    let amount = _a % 8;
    let signbit = 0x80 << (amount * 8);
    let index = (_a / 8) as usize;
    let part = b.0[index & 3] & signbit;
    let signmask64 = bitmask_bool(part > 0);
    let temp0 = blend_u64(signmask64, b.0[0], mask0);
    let temp1 = blend_u64(signmask64, b.0[1], mask1);
    let temp2 = blend_u64(signmask64, b.0[2], mask2);
    let temp3 = blend_u64(signmask64, b.0[3], mask3);
    let lt32 = bitmask_bool(is_ltpow2_u256(a, 32));
    let result0 = blend_u64(b.0[0], temp0, lt32);
    let result1 = blend_u64(b.0[1], temp1, lt32);
    let result2 = blend_u64(b.0[2], temp2, lt32);
    let result3 = blend_u64(b.0[3], temp3, lt32);
    U256([result0, result1, result2, result3])
}

#[allow(unreachable_code)]
pub unsafe fn eq_u256(a: U256, b: U256) -> U256 {
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
    // generic target
    let bit = (a.0[0] == b.0[0]) & (a.0[1] == b.0[1]) & (a.0[2] == b.0[2]) & (a.0[3] == b.0[3]);
    U256::from_bool(bit)
}

#[allow(unreachable_code)]
pub unsafe fn iszero_u256(a: U256) -> U256 {
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
    // generic target
    let bit = is_zero_u256(a);
    U256::from_bool(bit)
}

#[allow(unreachable_code)]
pub unsafe fn and_u256(a: U256, b: U256) -> U256 {
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
    // generic target
    U256([
        a.0[0] & b.0[0],
        a.0[1] & b.0[1],
        a.0[2] & b.0[2],
        a.0[3] & b.0[3],
    ])
}

#[allow(unreachable_code)]
pub unsafe fn or_u256(a: U256, b: U256) -> U256 {
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
    // generic target
    U256([
        a.0[0] | b.0[0],
        a.0[1] | b.0[1],
        a.0[2] | b.0[2],
        a.0[3] | b.0[3],
    ])
}

#[allow(unreachable_code)]
pub unsafe fn xor_u256(a: U256, b: U256) -> U256 {
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
    // generic target
    U256([
        a.0[0] ^ b.0[0],
        a.0[1] ^ b.0[1],
        a.0[2] ^ b.0[2],
        a.0[3] ^ b.0[3],
    ])
}

#[allow(unreachable_code)]
pub unsafe fn not_u256(value: U256) -> U256 {
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
    // generic target
    U256([!value.0[0], !value.0[1], !value.0[2], !value.0[3]])
}

#[allow(non_snake_case)]
const fn _MM_SHUFFLE(z: i32, y: i32, x: i32, w: i32) -> i32 {
    (z << 6) | (y << 4) | (x << 2) | w
}

#[cfg(target_feature = "ssse3")]
unsafe fn bshl_ssse3(value: (__m128i, __m128i), count: __m128i) -> (__m128i, __m128i) {
    let zero = _mm_setzero_si128();
    let sixteen = _mm_set_epi64x(0, 16);
    let lane8_id = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    // byte shift
    let bcount = _mm_shuffle_epi8(count, zero);
    let smask = _mm_sub_epi8(lane8_id, bcount);
    let templo = _mm_shuffle_epi8(value.0, smask);
    let temphi = _mm_shuffle_epi8(value.1, smask);
    // byte shift mask and carry
    let mask = _mm_cmplt_epi8(lane8_id, bcount);
    let icount = _mm_sub_epi8(sixteen, count);
    let bicount = _mm_shuffle_epi8(icount, zero);
    let csmask = _mm_add_epi8(lane8_id, bicount);
    let carry = _mm_shuffle_epi8(value.0, csmask);
    let resultlo = templo;
    let resulthi = _mm_or_si128(temphi, _mm_and_si128(carry, mask));
    return (resultlo, resulthi);
}

#[cfg(target_feature = "ssse3")]
unsafe fn bmask_ssse3(bcount: __m128i) -> (__m128i, __m128i) {
    let zero = _mm_setzero_si128();
    let lane8_id = _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let lane8_id_16 = _mm_add_epi8(lane8_id, _mm_shuffle_epi8(_mm_set_epi64x(0, 16), zero));
    let masklo = _mm_cmplt_epi8(lane8_id_16, bcount);
    let maskhi = _mm_cmplt_epi8(lane8_id, bcount);
    (masklo, maskhi)
}

#[inline(always)]
#[cfg(target_feature = "ssse3")]
unsafe fn bshrmsb_ssse3(
    value: (__m128i, __m128i),
    count: __m128i,
    arithmetic: bool,
) -> ((__m128i, __m128i), __m128i) {
    let zero = _mm_setzero_si128();
    let sixteen = _mm_set_epi64x(0, 16);
    let lane8_id = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let lane8_ceil0_id = _mm_add_epi8(lane8_id, _mm_shuffle_epi8(_mm_set_epi64x(0, 16 * 7), zero));
    let lane8_ceil1_id = _mm_add_epi8(lane8_id, _mm_shuffle_epi8(_mm_set_epi64x(0, 16 * 6), zero));
    // byte shift
    let bcount = _mm_shuffle_epi8(count, zero);
    let smask = _mm_add_epi8(lane8_ceil0_id, bcount);
    let templo = _mm_shuffle_epi8(value.0, smask);
    let temphi = _mm_shuffle_epi8(value.1, smask);
    // byte shift mask and carry
    let icount = _mm_sub_epi8(sixteen, count);
    let bicount = _mm_shuffle_epi8(icount, zero);
    let mask = _mm_cmplt_epi8(lane8_id, bicount);
    let csmask = _mm_add_epi8(lane8_ceil1_id, bcount);
    let carry = _mm_shuffle_epi8(value.1, csmask);
    let resulthi = temphi;
    let resultlo = _mm_or_si128(templo, _mm_andnot_si128(mask, carry));
    if arithmetic {
        let fifteen = _mm_set_epi64x(0, 15);
        let srav = _mm_srai_epi32(value.1, 31);
        let msb = _mm_shuffle_epi8(srav, _mm_shuffle_epi8(fifteen, zero));
        let (masklo, maskhi) = bmask_ssse3(bcount);
        let resultlo = mm_select_si128(resultlo, msb, masklo);
        let resulthi = mm_select_si128(resulthi, msb, maskhi);
        return ((resultlo, resulthi), msb);
    } else {
        return ((resultlo, resulthi), zero);
    };
}

#[inline(always)]
#[cfg(target_feature = "ssse3")]
unsafe fn bshr_ssse3(value: (__m128i, __m128i), count: __m128i) -> (__m128i, __m128i) {
    let (result, _) = bshrmsb_ssse3(value, count, false);
    result
}

#[allow(unreachable_code)]
pub unsafe fn shl_u256(count: U256, value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane32_id = _mm256_set_epi32(7, 6, 5, 4, 3, 2, 1, 0);
        let sixty_four = _mm_set_epi64x(0, 64);
        let one = _mm256_set_epi64x(0, 0, 0, 1);
        let max_u64 = _mm256_sub_epi64(_mm256_setzero_si256(), one);
        let max_u8 = _mm256_sub_epi8(_mm256_setzero_si256(), one);
        // word shift
        let count = std::mem::transmute::<U256, __m256i>(count);
        let value = std::mem::transmute::<U256, __m256i>(value);
        let count128 = _mm256_castsi256_si128(count);
        let co32 = _mm_srli_epi32(count128, 5);
        let bco32 = _mm256_broadcastd_epi32(co32);
        let pmask = _mm256_sub_epi32(lane32_id, bco32);
        let temp = _mm256_permutevar8x32_epi32(value, pmask);
        // word shift mask
        let mask = _mm256_cmpgt_epi32(bco32, lane32_id);
        let wordsl = _mm256_andnot_si256(mask, temp);
        // bit shift
        let slcount = _mm_sub_epi32(count128, _mm_slli_epi32(co32, 5));
        let srcount = _mm_sub_epi32(sixty_four, slcount);
        let sltemp = _mm256_sll_epi64(wordsl, slcount);
        let srtemp = _mm256_srl_epi64(wordsl, srcount);
        const PMASKI: i32 = _MM_SHUFFLE(2, 1, 0, 3);
        let carry = _mm256_andnot_si256(max_u64, _mm256_permute4x64_epi64(srtemp, PMASKI));
        let bitsl = _mm256_or_si256(sltemp, _mm256_andnot_si256(max_u64, carry));
        //
        let hi248 = _mm256_andnot_si256(max_u8, count);
        let hiisz = broadcast_avx2(is_zero_u256(hi248.as_u256()));
        let result = _mm256_and_si256(bitsl, hiisz);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let zero = _mm_setzero_si128();
        let one = _mm_set_epi64x(0, 1);
        let sixty_four = _mm_set_epi64x(0, 64);
        let max_u8 = _mm_sub_epi8(zero, one);
        // byte shift
        let count = std::mem::transmute::<U256, (__m128i, __m128i)>(count);
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let co8 = _mm_srli_epi32(count.0, 3);
        let (bytesllo, byteslhi) = bshl_ssse3(value, co8);
        // bit shift
        let slcount = _mm_sub_epi32(count.0, _mm_slli_epi32(co8, 3));
        let srcount = _mm_sub_epi32(sixty_four, slcount);
        let sltemplo = _mm_sll_epi64(bytesllo, slcount);
        let sltemphi = _mm_sll_epi64(byteslhi, slcount);
        let srtemplo = _mm_srl_epi64(bytesllo, srcount);
        let srtemphi = _mm_srl_epi64(byteslhi, srcount);
        let carrylo = _mm_bslli_si128(srtemplo, 8);
        let carryhi = _mm_unpacklo_epi64(_mm_bsrli_si128(srtemplo, 8), srtemphi);
        let bitsllo = _mm_or_si128(sltemplo, carrylo);
        let bitslhi = _mm_or_si128(sltemphi, carryhi);
        //
        let hi248 = (_mm_andnot_si128(max_u8, count.0), count.1);
        let hi248 = std::mem::transmute::<(__m128i, __m128i), U256>(hi248);
        let hiisz = broadcast_sse2(is_zero_u256(hi248));
        let result = (_mm_and_si128(hiisz, bitsllo), _mm_and_si128(hiisz, bitslhi));
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    // generic target
    let count64 = count.low_u64() & 0xff;
    let offset = (count64 / 64) as usize;
    let padded: [u64; 8] = [0, 0, 0, 0, value.0[0], value.0[1], value.0[2], value.0[3]];
    let wordsl: [u64; 4] = [
        padded[4 + 0 - offset],
        padded[4 + 1 - offset],
        padded[4 + 2 - offset],
        padded[4 + 3 - offset],
    ];
    let slcount = count64 % 64;
    let srcount = 64 - slcount;
    let sr0count = srcount.min(63);
    let sr1count = srcount - sr0count;
    let sltemp3 = wordsl[3] << slcount;
    let sltemp2 = wordsl[2] << slcount;
    let sltemp1 = wordsl[1] << slcount;
    let sltemp0 = wordsl[0] << slcount;
    let srtemp2 = (wordsl[2] >> sr0count) >> sr1count;
    let srtemp1 = (wordsl[1] >> sr0count) >> sr1count;
    let srtemp0 = (wordsl[0] >> sr0count) >> sr1count;
    let bitsl = U256([
        sltemp0,
        sltemp1 | srtemp0,
        sltemp2 | srtemp1,
        sltemp3 | srtemp2,
    ]);
    let hi248 = U256([count.0[0] & !0xff, count.0[1], count.0[2], count.0[3]]);
    let hiisz = U256::broadcast_u64(bitmask_bool(is_zero_u256(hi248)));
    let result = and_u256(bitsl, hiisz);
    result
}

#[inline(always)]
#[allow(unreachable_code)]
pub unsafe fn shr_u256(count: U256, value: U256, arithmetic: bool) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let lane32_id = _mm256_set_epi32(7, 6, 5, 4, 3, 2, 1, 0);
        let lane32_eight = _mm256_broadcastd_epi32(_mm_set_epi64x(0, 8));
        let sixty_four = _mm_set_epi64x(0, 64);
        let one = _mm256_set_epi64x(0, 0, 0, 1);
        let max_u64 = _mm256_set_epi64x(-1, 0, 0, 0);
        let max_u8 = _mm256_sub_epi8(_mm256_setzero_si256(), one);
        // word shift
        let count = std::mem::transmute::<U256, __m256i>(count);
        let value = std::mem::transmute::<U256, __m256i>(value);
        let count128 = _mm256_castsi256_si128(count);
        let co32 = _mm_srli_epi32(count128, 5);
        let bco32 = _mm256_broadcastd_epi32(co32);
        let pmask = _mm256_add_epi32(lane32_id, bco32);
        let temp = _mm256_permutevar8x32_epi32(value, pmask);
        // word shift mask
        let bico32 = _mm256_sub_epi32(lane32_eight, bco32);
        let mask = _mm256_cmpgt_epi32(bico32, lane32_id);
        let (wordsl, msb) = if arithmetic {
            let srav = _mm256_srai_epi32(value, 31);
            let msb = _mm256_permutevar8x32_epi32(srav, _mm256_set_epi32(7, 7, 7, 7, 7, 7, 7, 7));
            (_mm256_blendv_epi8(msb, temp, mask), msb)
        } else {
            (_mm256_and_si256(mask, temp), _mm256_setzero_si256())
        };
        // bit shift
        let srcount = _mm_sub_epi32(count128, _mm_slli_epi32(co32, 5));
        let slcount = _mm_sub_epi32(sixty_four, srcount);
        let srtemp = _mm256_srl_epi64(wordsl, srcount);
        let sltemp = _mm256_sll_epi64(wordsl, slcount);
        const PMASKI: i32 = _MM_SHUFFLE(0, 3, 2, 1);
        let carry0 = _mm256_andnot_si256(max_u64, _mm256_permute4x64_epi64(sltemp, PMASKI));
        let carry = if arithmetic {
            let slmsb = _mm256_sll_epi64(msb, slcount);
            _mm256_blendv_epi8(carry0, slmsb, max_u64) // TODO: check mask
        } else {
            carry0
        };
        let bitsl = _mm256_or_si256(srtemp, carry);
        //
        let hi248 = _mm256_andnot_si256(max_u8, count);
        let hiisz = broadcast_avx2(is_zero_u256(hi248.as_u256()));
        let result = if arithmetic {
            _mm256_blendv_epi8(msb, bitsl, hiisz)
        } else {
            _mm256_and_si256(bitsl, hiisz)
        };
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let zero = _mm_setzero_si128();
        let one = _mm_set_epi64x(0, 1);
        let sixty_four = _mm_set_epi64x(0, 64);
        let max_u8 = _mm_sub_epi8(zero, one);
        // byte shift
        let count = std::mem::transmute::<U256, (__m128i, __m128i)>(count);
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let co8 = _mm_srli_epi32(count.0, 3);
        let ((bytesrlo, bytesrhi), msb) = bshrmsb_ssse3(value, co8, arithmetic);
        // bit shift
        let srcount = _mm_sub_epi32(count.0, _mm_slli_epi32(co8, 3));
        let slcount = _mm_sub_epi32(sixty_four, srcount);
        let srtemplo = _mm_srl_epi64(bytesrlo, srcount);
        let srtemphi = _mm_srl_epi64(bytesrhi, srcount);
        let sltemplo = _mm_sll_epi64(bytesrlo, slcount);
        let sltemphi = _mm_sll_epi64(bytesrhi, slcount);
        let carrylo = _mm_unpacklo_epi64(_mm_bsrli_si128(sltemplo, 8), sltemphi);
        let carryhi = if arithmetic {
            let slmsb = _mm_sll_epi64(msb, slcount);
            _mm_unpackhi_epi64(sltemphi, slmsb)
        } else {
            _mm_bsrli_si128(sltemphi, 8)
        };
        let bitsllo = _mm_or_si128(srtemplo, carrylo);
        let bitslhi = _mm_or_si128(srtemphi, carryhi);
        //
        let hi248 = (_mm_andnot_si128(max_u8, count.0), count.1);
        let hi248 = std::mem::transmute::<(__m128i, __m128i), U256>(hi248);
        let hiisz = broadcast_sse2(is_zero_u256(hi248));
        let result = if arithmetic {
            (
                mm_blendv_epi8(msb, bitsllo, hiisz),
                mm_blendv_epi8(msb, bitslhi, hiisz),
            )
        } else {
            (_mm_and_si128(hiisz, bitsllo), _mm_and_si128(hiisz, bitslhi))
        };
        return std::mem::transmute::<(__m128i, __m128i), U256>(result);
    }
    // generic target
    let count64 = count.low_u64() & 0xff;
    let offset = (count64 / 64) as usize;
    let msb = if arithmetic {
        (-((value.0[3] >> 63) as i64)) as u64
    } else {
        0
    };
    let padded: [u64; 8] = [
        value.0[0], value.0[1], value.0[2], value.0[3], msb, msb, msb, msb,
    ];
    let wordsr: [u64; 4] = [
        padded[0 + offset],
        padded[1 + offset],
        padded[2 + offset],
        padded[3 + offset],
    ];
    let srcount = count64 % 64;
    let slcount = 64 - srcount;
    let sl0count = slcount.min(63);
    let sl1count = slcount - sl0count;
    let srtemp3 = wordsr[3] >> srcount;
    let srtemp2 = wordsr[2] >> srcount;
    let srtemp1 = wordsr[1] >> srcount;
    let srtemp0 = wordsr[0] >> srcount;
    let sltemp4 = (msb << sl0count) << sl1count;
    let sltemp3 = (wordsr[3] << sl0count) << sl1count;
    let sltemp2 = (wordsr[2] << sl0count) << sl1count;
    let sltemp1 = (wordsr[1] << sl0count) << sl1count;
    let bitsr = U256([
        srtemp0 | sltemp1,
        srtemp1 | sltemp2,
        srtemp2 | sltemp3,
        srtemp3 | sltemp4,
    ]);
    let hi248 = U256([count.0[0] & !0xff, count.0[1], count.0[2], count.0[3]]);
    let hiisz = U256::broadcast_u64(bitmask_bool(is_zero_u256(hi248)));
    if arithmetic {
        let result0 = blend_u64(msb, bitsr.0[0], hiisz.0[0]);
        let result1 = blend_u64(msb, bitsr.0[1], hiisz.0[1]);
        let result2 = blend_u64(msb, bitsr.0[2], hiisz.0[2]);
        let result3 = blend_u64(msb, bitsr.0[3], hiisz.0[3]);
        U256([result0, result1, result2, result3])
    } else {
        let result = and_u256(bitsr, hiisz);
        result
    }
}

pub fn overflowing_add_u256(a: U256, b: U256) -> (U256, bool) {
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

pub fn add_u256(a: U256, b: U256) -> U256 {
    let (value, _) = overflowing_add_u256(a, b);
    value
}

pub fn mul_u64(a: u64, b: u64) -> u128 {
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
        } else {
            let temp2 = temp + (carry as u128);
            if j == (num_limbs - 1) {
                r[j - 1] = temp2 as u64;
                r[j - 0] = (temp2 >> 64) as u64;
            } else {
                r[j - 1] = temp2 as u64;
                carry = (temp2 >> 64) as u64;
            }
        }
    }
}

fn mul_diagc(
    num_limbs: usize,
    i: usize,
    a: &[u64],
    b: u64,
    r: &mut [u64],
    rp: &mut [u64],
    c: &mut [u64],
) {
    let mut carry: u64 = 0;
    for j in 0..num_limbs {
        let temp = mul_u64(a[j], b) + (r[j] as u128);
        if j == 0 {
            c[i] = temp as u64;
            carry = (temp >> 64) as u64;
        } else {
            let temp2 = temp + (carry as u128);
            if j == (num_limbs - 1) {
                rp[j - 1] = temp2 as u64;
                rp[j - 0] = (temp2 >> 64) as u64;
            } else {
                rp[j - 1] = temp2 as u64;
                carry = (temp2 >> 64) as u64;
            }
        }
    }
}

fn mul_limbs(num_limbs: usize, a: &[u64], b: &[u64], c: &mut [u64]) {
    debug_assert!(num_limbs <= 4);
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
        c[num_limbs + i] = rp[i];
    }
}

pub fn mul_u256(a: U256, b: U256) -> U256 {
    let mut c: [u64; 8] = unsafe { std::mem::uninitialized() };
    mul_limbs(4, &a.0, &b.0, &mut c);
    U256([c[0], c[1], c[2], c[3]])
}

pub fn fullmul_u256(a: U256, b: U256) -> U512 {
    let mut c: [u64; 8] = unsafe { std::mem::uninitialized() };
    mul_limbs(4, &a.0, &b.0, &mut c);
    U512(c)
}

fn overflowing_sub_u256(a: U256, b: U256) -> (U256, bool) {
    let alo = ((a.0[1] as u128) << 64) | (a.0[0] as u128);
    let blo = ((b.0[1] as u128) << 64) | (b.0[0] as u128);
    let ahi = ((a.0[3] as u128) << 64) | (a.0[2] as u128);
    let bhi = ((b.0[3] as u128) << 64) | (b.0[2] as u128);
    let (lo, borrowlo) = alo.overflowing_sub(blo);
    let hi = ahi.wrapping_sub(bhi).wrapping_sub(borrowlo as u128);
    let borrow = (ahi < bhi) | ((ahi == bhi) & borrowlo);
    (
        U256([lo as u64, (lo >> 64) as u64, hi as u64, (hi >> 64) as u64]),
        borrow,
    )
}

pub fn sub_u256(a: U256, b: U256) -> U256 {
    let (value, _) = overflowing_sub_u256(a, b);
    value
}

pub fn lt_u256(a: U256, b: U256) -> bool {
    let alo = a.low_u128();
    let blo = b.low_u128();
    let ahi = a.high_u128();
    let bhi = b.high_u128();
    (ahi < bhi) | ((ahi == bhi) & (alo < blo))
}

pub fn gt_u256(a: U256, b: U256) -> bool {
    let alo = a.low_u128();
    let blo = b.low_u128();
    let ahi = a.high_u128();
    let bhi = b.high_u128();
    (ahi > bhi) | ((ahi == bhi) & (alo > blo))
}

pub fn slt_u256(a: U256, b: U256) -> bool {
    let alo = a.low_u128();
    let blo = b.low_u128();
    let ahi = a.high_u128();
    let bhi = b.high_u128();
    let ahis = ahi as i128;
    let bhis = bhi as i128;
    (ahis < bhis) | ((ahi == bhi) & (alo < blo))
}

pub fn sgt_u256(a: U256, b: U256) -> bool {
    let alo = a.low_u128();
    let blo = b.low_u128();
    let ahi = a.high_u128();
    let bhi = b.high_u128();
    let ahis = ahi as i128;
    let bhis = bhi as i128;
    (ahis > bhis) | ((ahi == bhi) & (alo > blo))
}

#[allow(unreachable_code)]
unsafe fn move_mask(value: U256) -> u32 {
    #[cfg(target_feature = "avx2")]
    {
        let value = std::mem::transmute::<U256, __m256i>(value);
        let temp = _mm256_cmpeq_epi32(value, _mm256_setzero_si256());
        let bits = _mm256_movemask_epi8(temp);
        return bits as u32;
    }
    #[cfg(target_feature = "ssse3")]
    {
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let templo = _mm_cmpeq_epi32(value.0, _mm_setzero_si128());
        let temphi = _mm_cmpeq_epi32(value.1, _mm_setzero_si128());
        let bits = _mm_movemask_epi8(templo) | (_mm_movemask_epi8(temphi) << 16);
        return bits as u32;
    }
    // generic target
    (((((value.0[0]) as u32) == 0) as u32) * (0xf))
        | (((((value.0[0] >> 32) as u32) == 0) as u32) * (0xf << 4))
        | (((((value.0[1]) as u32) == 0) as u32) * (0xf << 8))
        | (((((value.0[1] >> 32) as u32) == 0) as u32) * (0xf << 12))
        | (((((value.0[2]) as u32) == 0) as u32) * (0xf << 16))
        | (((((value.0[2] >> 32) as u32) == 0) as u32) * (0xf << 20))
        | (((((value.0[3]) as u32) == 0) as u32) * (0xf << 24))
        | (((((value.0[3] >> 32) as u32) == 0) as u32) * (0xf << 28))
}

pub fn count_u32s(value: U256) -> isize {
    let bits = unsafe { move_mask(value) };
    let inv: u64 = !((bits as u64) << 32);
    let clz = inv.leading_zeros();
    let count = 8 - clz / 4;
    count as isize
}

#[allow(unreachable_code)]
pub unsafe fn negate_u256(value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        const LUT: [U256; 8] = [
            U256([1, 0, 0, 0]), // 0: 000
            U256([1, 1, 0, 0]), // 1: 001
            U256([1, 0, 0, 0]), // 2: 010
            U256([1, 1, 1, 0]), // 3: 011
            U256([1, 0, 0, 0]), // 4: 100
            U256([1, 1, 0, 0]), // 5: 101
            U256([1, 0, 0, 0]), // 6: 110
            U256([1, 1, 1, 1]), // 7: 111
        ];
        let all_ones = _mm256_set_epi64x(-1, -1, -1, -1);
        //
        let value = std::mem::transmute::<U256, __m256i>(value);
        let notv = _mm256_andnot_si256(value, all_ones);
        let mask = _mm256_cmpeq_epi64(notv, all_ones);
        let bits = _mm256_movemask_epi8(mask);
        let index = _pext_u32(bits as u32, (1 | (1 << 8) | (1 << 16)));
        let ptr = std::mem::transmute::<_, *const __m256i>(&LUT[0]);
        let carry = _mm256_load_si256(ptr.offset(index as isize));
        let result = _mm256_add_epi64(notv, carry);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let all_ones = _mm_set_epi64x(-1, -1);
        let one = _mm_set_epi64x(0, 1);
        let max_u64 = _mm_sub_epi64(_mm_setzero_si128(), one);
        let lane64_one = _mm_shuffle_epi32(one, _MM_SHUFFLE(1, 0, 1, 0));
        //
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let notvlo = _mm_andnot_si128(value.0, all_ones);
        let notvhi = _mm_andnot_si128(value.1, all_ones);
        let mask0lo = mm_cmpeq_epi64(all_ones, notvlo);
        let mask0hi = mm_cmpeq_epi64(all_ones, notvhi);
        let mask1lo = _mm_or_si128(_mm_bslli_si128(mask0lo, 8), max_u64);
        let mask1hi = _mm_unpackhi_epi64(mask0lo, _mm_bslli_si128(mask0hi, 8));
        let mask3hi = _mm_or_si128(_mm_bslli_si128(mask0lo, 8), max_u64);
        let maskhi = _mm_and_si128(_mm_and_si128(mask1hi, mask0lo), mask3hi);
        let carrylo = _mm_and_si128(mask1lo, lane64_one);
        let carryhi = _mm_and_si128(maskhi, lane64_one);
        let resultlo = _mm_add_epi64(notvlo, carrylo);
        let resulthi = _mm_add_epi64(notvhi, carryhi);
        return std::mem::transmute::<(__m128i, __m128i), U256>((resultlo, resulthi));
    }
    // generic target
    add_u256(not_u256(value), U256::from_u64(1))
}

#[allow(unreachable_code)]
pub unsafe fn is_neg_u256(value: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let value = std::mem::transmute::<U256, __m256i>(value);
        let temp = _mm256_srai_epi32(value, 31);
        let result = _mm256_permutevar8x32_epi32(temp, _mm256_set_epi32(7, 7, 7, 7, 7, 7, 7, 7));
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let value = std::mem::transmute::<U256, (__m128i, __m128i)>(value);
        let temp = _mm_srai_epi32(value.1, 31);
        let result = _mm_shuffle_epi32(temp, _MM_SHUFFLE(3, 3, 3, 3));
        return std::mem::transmute::<(__m128i, __m128i), U256>((result, result));
    }
    // generic target
    let temp = ((value.0[3] as i64) >> 63) as u64;
    return U256::broadcast_u64(temp);
}

#[allow(unreachable_code)]
pub unsafe fn opposite_signs_u256(a: U256, b: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let a = std::mem::transmute::<U256, __m256i>(a);
        let b = std::mem::transmute::<U256, __m256i>(b);
        let temp = _mm256_xor_si256(a, b);
        return is_neg_u256(std::mem::transmute::<__m256i, U256>(temp));
    }
    #[cfg(target_feature = "ssse3")]
    {
        let a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let temp = _mm_xor_si128(a.1, b.1);
        return is_neg_u256(std::mem::transmute::<(__m128i, __m128i), U256>((
            _mm_undefined_si128(),
            temp,
        )));
    }
    // generic target
    let temp = a.0[3] ^ b.0[3];
    return is_neg_u256(U256::broadcast_u64(temp));
}

#[allow(unreachable_code)]
pub unsafe fn blend_u256(a: U256, b: U256, mask: U256) -> U256 {
    #[cfg(target_feature = "avx2")]
    {
        let a = std::mem::transmute::<U256, __m256i>(a);
        let b = std::mem::transmute::<U256, __m256i>(b);
        let mask = std::mem::transmute::<U256, __m256i>(mask);
        let result = _mm256_blendv_epi8(a, b, mask);
        return std::mem::transmute::<__m256i, U256>(result);
    }
    #[cfg(target_feature = "ssse3")]
    {
        let a = std::mem::transmute::<U256, (__m128i, __m128i)>(a);
        let b = std::mem::transmute::<U256, (__m128i, __m128i)>(b);
        let mask = std::mem::transmute::<U256, (__m128i, __m128i)>(mask);
        let resultlo = mm_blendv_epi8(a.0, b.0, mask.0);
        let resulthi = mm_blendv_epi8(a.1, b.1, mask.1);
        return std::mem::transmute::<(__m128i, __m128i), U256>((resultlo, resulthi));
    }
    // generic target
    let result0 = blend_u64(a.0[0], b.0[0], mask.0[0]);
    let result1 = blend_u64(a.0[1], b.0[1], mask.0[1]);
    let result2 = blend_u64(a.0[2], b.0[2], mask.0[2]);
    let result3 = blend_u64(a.0[3], b.0[3], mask.0[3]);
    U256([result0, result1, result2, result3])
}

pub unsafe fn abs_u256(value: U256) -> U256 {
    let negv = negate_u256(value);
    let isneg = is_neg_u256(value);
    blend_u256(value, negv, isneg)
}

pub fn leading_zeros_u256(value: U256) -> usize {
    let mask3 = -((value.0[3] == 0) as i64) as u64;
    let mask2 = -((value.0[2] == 0) as i64) as u64;
    let mask1 = -((value.0[1] == 0) as i64) as u64;
    let count = (value.0[3].leading_zeros() as u64)
        + (value.0[2].leading_zeros() as u64 & mask3)
        + (value.0[1].leading_zeros() as u64 & mask3 & mask2)
        + (value.0[0].leading_zeros() as u64 & mask3 & mask2 & mask1);
    return count as usize;
}

// Knuth's Algorithm D from Hacker's Delight
pub unsafe fn divmnu(u: *const u32, v: *const u32, m: isize, n: isize, q: *mut u32, r: *mut u32) {
    debug_assert!(m <= 16);
    debug_assert!(n <= 8);
    let b: u64 = 1 << 32;
    if m < n {
        for i in 0..8 {
            *r.offset(i) = *u.offset(i);
        }
        return;
    }
    if n <= 0 || (*v.offset(n - 1) == 0) {
        return;
    }
    if n == 1 {
        let mut k = 0;
        let mut j = m - 1;
        while j >= 0 {
            let temp = *v as u64;
            *q.offset(j) = ((k * b + *u.offset(j) as u64) / temp) as u32;
            k = (k * b + *u.offset(j) as u64) - (*q.offset(j) as u64 * temp);
            j -= 1;
        }
        *r = k as u32;
        return;
    }
    //
    debug_assert!(*v.offset(n - 1) != 0);
    let s = (*v.offset(n - 1)).leading_zeros();
    let mut undata: [u32; 16 + 1] = MaybeUninit::uninit().assume_init();
    let un = &mut undata[0] as *mut u32;
    *un.offset(m) = (*u.offset(m - 1) as u64 >> (32 - s)) as u32;
    let mut i = m - 1;
    while i > 0 {
        *un.offset(i) = ((*u.offset(i) << s) as u64 | (*u.offset(i - 1) as u64 >> (32 - s))) as u32;
        i -= 1;
    }
    *un = *u << s;
    let mut vndata: [u32; 8] = MaybeUninit::uninit().assume_init();
    let vn = &mut vndata[0] as *mut u32;
    let mut i = n - 1;
    while i > 0 {
        *vn.offset(i) = ((*v.offset(i) << s) as u64 | (*v.offset(i - 1) as u64 >> (32 - s))) as u32;
        i -= 1;
    }
    *vn = *v << s;
    //
    let mut qhat: u64 = MaybeUninit::uninit().assume_init();
    let mut rhat: u64 = MaybeUninit::uninit().assume_init();
    let mut j = m - n;
    while j >= 0 {
        let temp0 = *un.offset(j + n) as u64 * b + *un.offset(j + n - 1) as u64;
        let temp1 = *vn.offset(n - 1) as u64;
        qhat = temp0 / temp1;
        rhat = temp0 - qhat * temp1;
        loop {
            if (qhat >= b)
                | ((qhat * *vn.offset(n - 2) as u64) > (b * rhat + un.offset(j + n - 2) as u64))
            {
                qhat -= 1;
                rhat += *vn.offset(n - 1) as u64;
                if rhat < b {
                    continue;
                }
            }
            break;
        }
        let mut k: u64 = 0;
        let mut t: i64 = MaybeUninit::uninit().assume_init();
        for i in 0..n {
            let p = qhat * *vn.offset(i) as u64;
            t = {
                let temp = (*un.offset(i + j) as u64).wrapping_sub(k);
                temp.wrapping_sub(p & (u32::max_value() as u64))
            } as i64;
            *un.offset(i + j) = t as u32;
            k = (p >> 32).wrapping_sub((t >> 32) as u64);
        }
        t = (*un.offset(j + n) as u64).wrapping_sub(k) as i64;
        *un.offset(j + n) = t as u32;
        //
        *q.offset(j) = qhat as u32;
        if t < 0 {
            *q.offset(j) = *q.offset(j) - 1;
            k = 0;
            for i in 0..n {
                t = (*un.offset(i + j) as u64 + *vn.offset(i) as u64 + k) as i64;
                *un.offset(i + j) = t as u32;
                k = (t >> 32) as u64;
            }
            *un.offset(j + n) = (*un.offset(j + n) as u64 + k) as u32;
        }
        //
        for i in 0..n - 1 {
            *r.offset(i) =
                ((*un.offset(i) >> s) as u64 | ((*un.offset(i + 1) as u64) << (32 - s))) as u32;
        }
        *r.offset(n - 1) = *un.offset(n - 1) >> s;
        j -= 1;
    }
}

pub unsafe fn div_u256(a: U256, b: U256) -> U256 {
    let u = std::mem::transmute::<_, *const u32>(&a.0[0]);
    let v = std::mem::transmute::<_, *const u32>(&b.0[0]);
    let m = count_u32s(a);
    let n = count_u32s(b);
    let mut qv = U256::default();
    let mut rv = U256::default();
    let q = std::mem::transmute::<_, *mut u32>(&mut qv.0[0]);
    let r = std::mem::transmute::<_, *mut u32>(&mut rv.0[0]);
    divmnu(u, v, m, n, q, r);
    qv
}

pub unsafe fn mod_u256(a: U256, b: U256) -> U256 {
    let u = std::mem::transmute::<_, *const u32>(&a.0[0]);
    let v = std::mem::transmute::<_, *const u32>(&b.0[0]);
    let m = count_u32s(a);
    let n = count_u32s(b);
    let mut qv = U256::default();
    let mut rv = U256::default();
    let q = std::mem::transmute::<_, *mut u32>(&mut qv.0[0]);
    let r = std::mem::transmute::<_, *mut u32>(&mut rv.0[0]);
    divmnu(u, v, m, n, q, r);
    rv
}

pub unsafe fn sdiv_u256(a: U256, b: U256) -> U256 {
    let absa = abs_u256(a);
    let absb = abs_u256(b);
    let q = div_u256(absa, absb);
    let negq = negate_u256(q);
    blend_u256(q, negq, opposite_signs_u256(a, b))
}

pub unsafe fn smod_u256(a: U256, b: U256) -> U256 {
    let absa = abs_u256(a);
    let absb = abs_u256(b);
    let r = mod_u256(absa, absb);
    let negr = negate_u256(r);
    blend_u256(r, negr, opposite_signs_u256(a, b))
}

pub unsafe fn addmod_u256(a: U256, b: U256, c: U256) -> U256 {
    let (temp, overflow) = overflowing_add_u256(a, b);
    let s: [u32; 9] = [
        (temp.0[0] as u32),
        (temp.0[0] >> 32) as u32,
        (temp.0[1] as u32),
        (temp.0[1] >> 32) as u32,
        (temp.0[2] as u32),
        (temp.0[2] >> 32) as u32,
        (temp.0[3] as u32),
        (temp.0[3] >> 32) as u32,
        overflow as u32,
    ];
    let u = &s[0] as *const u32;
    let v = std::mem::transmute::<_, *const u32>(&c.0[0]);
    let am = count_u32s(a);
    let bm = count_u32s(b);
    let m = am.max(bm) + overflow as isize;
    let n = count_u32s(c);
    let mut qv = U256::default();
    let mut rv = U256::default();
    let q = std::mem::transmute::<_, *mut u32>(&mut qv.0[0]);
    let r = std::mem::transmute::<_, *mut u32>(&mut rv.0[0]);
    divmnu(u, v, m, n, q, r);
    rv
}

pub unsafe fn mulmod_u256(a: U256, b: U256, c: U256) -> U256 {
    let p = fullmul_u256(a, b);
    let u = std::mem::transmute::<_, *const u32>(&p.0[0]);
    let v = std::mem::transmute::<_, *const u32>(&c.0[0]);
    let am = count_u32s(a);
    let bm = count_u32s(b);
    let m = am + bm;
    let n = count_u32s(c);
    let mut qv = U256::default();
    let mut rv = U256::default();
    let q = std::mem::transmute::<_, *mut u32>(&mut qv.0[0]);
    let r = std::mem::transmute::<_, *mut u32>(&mut rv.0[0]);
    divmnu(u, v, m, n, q, r);
    rv
}

pub fn exp_u256(base: U256, exponent: U256, exponent_bits: usize) -> U256 {
    let mut result = U256::from_u64(1);
    let mut acc = base;
    let mut i = 0;
    loop {
        let bit_on = exponent.0[i / 64] & (1 << (i % 64));
        if bit_on > 0 {
            result = mul_u256(result, acc);
        }
        i += 1;
        if i < exponent_bits {
            acc = mul_u256(acc, acc);
            continue;
        }
        break;
    }
    result
}

#[cfg(target_feature = "sse4.1")]
macro_rules! mm_extract_epi64 {
    ($a:expr, 0) => {
        _mm_cvtsi128_si64($a)
    };
    ($a:expr, 1) => {
        _mm_extract_epi64($a, 1)
    }
}

#[cfg(all(not(target_feature = "sse4.1"), target_feature = "sse2"))]
macro_rules! mm_extract_epi64 {
    ($a:expr, 0) => {
        _mm_cvtsi128_si64($a)
    };
    ($a:expr, 1) => {
        _mm_cvtsi128_si64(_mm_srli_si128($a, 8))
    }
}

fn rol_u64(a: u64, b: u64) -> u64 {
    (a << b) | (a >> (64 - b))
}

fn andn_u64(a: u64, b: u64) -> u64 {
    !a & b
}

// From Markku-Juhani O. Saarinen
fn keccakf1600(state: &mut [u64; 25]) {
    const KECCAKF_RNDC: [u64; 24] = [
        0x0000000000000001,
        0x0000000000008082,
        0x800000000000808a,
        0x8000000080008000,
        0x000000000000808b,
        0x0000000080000001,
        0x8000000080008081,
        0x8000000000008009,
        0x000000000000008a,
        0x0000000000000088,
        0x0000000080008009,
        0x000000008000000a,
        0x000000008000808b,
        0x800000000000008b,
        0x8000000000008089,
        0x8000000000008003,
        0x8000000000008002,
        0x8000000000000080,
        0x000000000000800a,
        0x800000008000000a,
        0x8000000080008081,
        0x8000000000008080,
        0x0000000080000001,
        0x8000000080008008,
    ];
    const KECCAKF_ROTC: [u64; 24] = [
        1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
    ];
    const KECCAKF_PILN: [usize; 24] = [
        10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
    ];
    let mut bc: [u64; 5] = unsafe { MaybeUninit::uninit().assume_init() };
    for r in 0..24 {
        // theta
        for i in 0..5 {
            bc[i] = state[i] ^ state[i + 5] ^ state[i + 10] ^ state[i + 15] ^ state[i + 20];
        }
        for i in 0..5 {
            let t = bc[(i + 4) % 5] ^ rol_u64(bc[(i + 1) % 5], 1);
            for j in 0..5 {
                let j = j * 5;
                state[j + i] ^= t;
            }
        }
        // rho and pi
        let mut t = state[1];
        for i in 0..12 {
            let j = KECCAKF_PILN[i];
            bc[0] = state[j];
            state[j as usize] = rol_u64(t, KECCAKF_ROTC[i]);
            t = bc[0];
        }
        for i in 12..24 {
            let j = KECCAKF_PILN[i];
            bc[0] = state[j];
            state[j as usize] = rol_u64(t, KECCAKF_ROTC[i]);
            t = bc[0];
        }
        // chi
        for j in 0..5 {
            let j = j * 5;
            for i in 0..5 {
                bc[i] = state[j + i];
            }
            for i in 0..5 {
                state[j + i] ^= andn_u64(bc[(i + 1) % 5], bc[(i + 2) % 5]);
            }
        }
        // iota
        state[0] ^= KECCAKF_RNDC[r];
    }
}

pub unsafe fn keccak256(input: *const u8, size: usize) -> U256 {
    const RATE: usize = 200 - 2 * 32;
    let mut input = input;
    let mut size = size;
    let mut state: [u64; 25] = [0; 25];
    while size >= RATE {
        for i in 0..(RATE / 8) {
            state[i] ^= *(input as *const u64).offset(i as isize);
        }
        keccakf1600(&mut state);
        input = input.offset(RATE as isize);
        size -= RATE;
    }
    let mut temp: [u8; 136] = [0; 136];
    let ptr = temp.as_mut_ptr();
    for i in 0..size {
        *ptr.offset(i as isize) = *input.offset(i as isize);
    }
    temp[size] = 0x01;
    temp[RATE - 1] |= 0x80;
    let ptr = temp.as_mut_ptr() as *const u64;
    for i in 0..(RATE / 8) {
        state[i] ^= *ptr.offset(i as isize);
    }
    keccakf1600(&mut state);
    U256([state[0], state[1], state[2], state[3]])
}

pub unsafe fn sha3_u256(input: *const u8, size: usize) -> U256 {
    let hash = keccak256(input, size);
    let result = bswap_u256(hash);
    result
}

#[cfg(target_feature = "sse2")]
fn assert_word_eq(a: (__m128i, __m128i), b: (__m128i, __m128i)) {
    unsafe {
        assert_eq!(mm_extract_epi64!(a.0, 0), mm_extract_epi64!(b.0, 0));
        assert_eq!(mm_extract_epi64!(a.0, 1), mm_extract_epi64!(b.0, 1));
        assert_eq!(mm_extract_epi64!(a.1, 0), mm_extract_epi64!(b.1, 0));
        assert_eq!(mm_extract_epi64!(a.1, 1), mm_extract_epi64!(b.1, 1));
    }
}

#[cfg(target_feature = "ssse3")]
#[test]
fn test_bshl_ssse3() {
    unsafe {
        let i = (
            _mm_set_epi64x(0xc0c1c2c3c4c5c6c7u64 as i64, 0xd0d1d2d3d4d5d6d7u64 as i64),
            _mm_set_epi64x(0xa0a1a2a3a4a5a6a7u64 as i64, 0xb0b1b2b3b4b5b6b7u64 as i64),
        );
        let o = (
            _mm_set_epi64x(0xc0c1c2c3c4c5c6c7u64 as i64, 0xd0d1d2d3d4d5d6d7u64 as i64),
            _mm_set_epi64x(0xa0a1a2a3a4a5a6a7u64 as i64, 0xb0b1b2b3b4b5b6b7u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 0)), o);
        let o = (
            _mm_set_epi64x(0xc1c2c3c4c5c6c7d0u64 as i64, 0xd1d2d3d4d5d6d700u64 as i64),
            _mm_set_epi64x(0xa1a2a3a4a5a6a7b0u64 as i64, 0xb1b2b3b4b5b6b7c0u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 1)), o);
        let o = (
            _mm_set_epi64x(0xc2c3c4c5c6c7d0d1u64 as i64, 0xd2d3d4d5d6d70000u64 as i64),
            _mm_set_epi64x(0xa2a3a4a5a6a7b0b1u64 as i64, 0xb2b3b4b5b6b7c0c1u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 2)), o);
        let o = (
            _mm_set_epi64x(0xc3c4c5c6c7d0d1d2u64 as i64, 0xd3d4d5d6d7000000u64 as i64),
            _mm_set_epi64x(0xa3a4a5a6a7b0b1b2u64 as i64, 0xb3b4b5b6b7c0c1c2u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 3)), o);
        let o = (
            _mm_set_epi64x(0xc4c5c6c7d0d1d2d3u64 as i64, 0xd4d5d6d700000000u64 as i64),
            _mm_set_epi64x(0xa4a5a6a7b0b1b2b3u64 as i64, 0xb4b5b6b7c0c1c2c3u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 4)), o);
        let o = (
            _mm_set_epi64x(0xc5c6c7d0d1d2d3d4u64 as i64, 0xd5d6d70000000000u64 as i64),
            _mm_set_epi64x(0xa5a6a7b0b1b2b3b4u64 as i64, 0xb5b6b7c0c1c2c3c4u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 5)), o);
        let o = (
            _mm_set_epi64x(0xc6c7d0d1d2d3d4d5u64 as i64, 0xd6d7000000000000u64 as i64),
            _mm_set_epi64x(0xa6a7b0b1b2b3b4b5u64 as i64, 0xb6b7c0c1c2c3c4c5u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 6)), o);
        let o = (
            _mm_set_epi64x(0xc7d0d1d2d3d4d5d6u64 as i64, 0xd700000000000000u64 as i64),
            _mm_set_epi64x(0xa7b0b1b2b3b4b5b6u64 as i64, 0xb7c0c1c2c3c4c5c6u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 7)), o);
        let o = (
            _mm_set_epi64x(0xd0d1d2d3d4d5d6d7u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb0b1b2b3b4b5b6b7u64 as i64, 0xc0c1c2c3c4c5c6c7u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 8)), o);
        let o = (
            _mm_set_epi64x(0xd1d2d3d4d5d6d700u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb1b2b3b4b5b6b7c0u64 as i64, 0xc1c2c3c4c5c6c7d0u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 9)), o);
        let o = (
            _mm_set_epi64x(0xd2d3d4d5d6d70000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb2b3b4b5b6b7c0c1u64 as i64, 0xc2c3c4c5c6c7d0d1u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 10)), o);
        let o = (
            _mm_set_epi64x(0xd3d4d5d6d7000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb3b4b5b6b7c0c1c2u64 as i64, 0xc3c4c5c6c7d0d1d2u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 11)), o);
        let o = (
            _mm_set_epi64x(0xd4d5d6d700000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb4b5b6b7c0c1c2c3u64 as i64, 0xc4c5c6c7d0d1d2d3u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 12)), o);
        let o = (
            _mm_set_epi64x(0xd5d6d70000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb5b6b7c0c1c2c3c4u64 as i64, 0xc5c6c7d0d1d2d3d4u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 13)), o);
        let o = (
            _mm_set_epi64x(0xd6d7000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb6b7c0c1c2c3c4c5u64 as i64, 0xc6c7d0d1d2d3d4d5u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 14)), o);
        let o = (
            _mm_set_epi64x(0xd700000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xb7c0c1c2c3c4c5c6u64 as i64, 0xc7d0d1d2d3d4d5d6u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 15)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc0c1c2c3c4c5c6c7u64 as i64, 0xd0d1d2d3d4d5d6d7u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 16)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc1c2c3c4c5c6c7d0u64 as i64, 0xd1d2d3d4d5d6d700u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 17)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc2c3c4c5c6c7d0d1u64 as i64, 0xd2d3d4d5d6d70000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 18)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc3c4c5c6c7d0d1d2u64 as i64, 0xd3d4d5d6d7000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 19)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc4c5c6c7d0d1d2d3u64 as i64, 0xd4d5d6d700000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 20)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc5c6c7d0d1d2d3d4u64 as i64, 0xd5d6d70000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 21)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc6c7d0d1d2d3d4d5u64 as i64, 0xd6d7000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 22)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xc7d0d1d2d3d4d5d6u64 as i64, 0xd700000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 23)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd0d1d2d3d4d5d6d7u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 24)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd1d2d3d4d5d6d700u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 25)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd2d3d4d5d6d70000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 26)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd3d4d5d6d7000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 27)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd4d5d6d700000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 28)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd5d6d70000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 29)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd6d7000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 30)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
            _mm_set_epi64x(0xd700000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshl_ssse3(i, _mm_set_epi64x(0, 31)), o);
    }
}

#[cfg(target_feature = "ssse3")]
#[test]
fn test_bshr_ssse3() {
    unsafe {
        let i = (
            _mm_set_epi64x(0xc0c1c2c3c4c5c6c7u64 as i64, 0xd0d1d2d3d4d5d6d7u64 as i64),
            _mm_set_epi64x(0xa0a1a2a3a4a5a6a7u64 as i64, 0xb0b1b2b3b4b5b6b7u64 as i64),
        );
        let o = (
            _mm_set_epi64x(0xc0c1c2c3c4c5c6c7u64 as i64, 0xd0d1d2d3d4d5d6d7u64 as i64),
            _mm_set_epi64x(0xa0a1a2a3a4a5a6a7u64 as i64, 0xb0b1b2b3b4b5b6b7u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 0)), o);
        let o = (
            _mm_set_epi64x(0xb7c0c1c2c3c4c5c6u64 as i64, 0xc7d0d1d2d3d4d5d6u64 as i64),
            _mm_set_epi64x(0x00a0a1a2a3a4a5a6u64 as i64, 0xa7b0b1b2b3b4b5b6u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 1)), o);
        let o = (
            _mm_set_epi64x(0xb6b7c0c1c2c3c4c5u64 as i64, 0xc6c7d0d1d2d3d4d5u64 as i64),
            _mm_set_epi64x(0x0000a0a1a2a3a4a5u64 as i64, 0xa6a7b0b1b2b3b4b5u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 2)), o);
        let o = (
            _mm_set_epi64x(0xb5b6b7c0c1c2c3c4u64 as i64, 0xc5c6c7d0d1d2d3d4u64 as i64),
            _mm_set_epi64x(0x000000a0a1a2a3a4u64 as i64, 0xa5a6a7b0b1b2b3b4u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 3)), o);
        let o = (
            _mm_set_epi64x(0xb4b5b6b7c0c1c2c3u64 as i64, 0xc4c5c6c7d0d1d2d3u64 as i64),
            _mm_set_epi64x(0x00000000a0a1a2a3u64 as i64, 0xa4a5a6a7b0b1b2b3u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 4)), o);
        let o = (
            _mm_set_epi64x(0xb3b4b5b6b7c0c1c2u64 as i64, 0xc3c4c5c6c7d0d1d2u64 as i64),
            _mm_set_epi64x(0x0000000000a0a1a2u64 as i64, 0xa3a4a5a6a7b0b1b2u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 5)), o);
        let o = (
            _mm_set_epi64x(0xb2b3b4b5b6b7c0c1u64 as i64, 0xc2c3c4c5c6c7d0d1u64 as i64),
            _mm_set_epi64x(0x000000000000a0a1u64 as i64, 0xa2a3a4a5a6a7b0b1u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 6)), o);
        let o = (
            _mm_set_epi64x(0xb1b2b3b4b5b6b7c0u64 as i64, 0xc1c2c3c4c5c6c7d0u64 as i64),
            _mm_set_epi64x(0x00000000000000a0u64 as i64, 0xa1a2a3a4a5a6a7b0u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 7)), o);
        let o = (
            _mm_set_epi64x(0xb0b1b2b3b4b5b6b7u64 as i64, 0xc0c1c2c3c4c5c6c7u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0xa0a1a2a3a4a5a6a7u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 8)), o);
        let o = (
            _mm_set_epi64x(0xa7b0b1b2b3b4b5b6u64 as i64, 0xb7c0c1c2c3c4c5c6u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x00a0a1a2a3a4a5a6u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 9)), o);
        let o = (
            _mm_set_epi64x(0xa6a7b0b1b2b3b4b5u64 as i64, 0xb6b7c0c1c2c3c4c5u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000a0a1a2a3a4a5u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 10)), o);
        let o = (
            _mm_set_epi64x(0xa5a6a7b0b1b2b3b4u64 as i64, 0xb5b6b7c0c1c2c3c4u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x000000a0a1a2a3a4u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 11)), o);
        let o = (
            _mm_set_epi64x(0xa4a5a6a7b0b1b2b3u64 as i64, 0xb4b5b6b7c0c1c2c3u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x00000000a0a1a2a3u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 12)), o);
        let o = (
            _mm_set_epi64x(0xa3a4a5a6a7b0b1b2u64 as i64, 0xb3b4b5b6b7c0c1c2u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000a0a1a2u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 13)), o);
        let o = (
            _mm_set_epi64x(0xa2a3a4a5a6a7b0b1u64 as i64, 0xb2b3b4b5b6b7c0c1u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x000000000000a0a1u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 14)), o);
        let o = (
            _mm_set_epi64x(0xa1a2a3a4a5a6a7b0u64 as i64, 0xb1b2b3b4b5b6b7c0u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x00000000000000a0u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 15)), o);
        let o = (
            _mm_set_epi64x(0xa0a1a2a3a4a5a6a7u64 as i64, 0xb0b1b2b3b4b5b6b7u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 16)), o);
        let o = (
            _mm_set_epi64x(0x00a0a1a2a3a4a5a6u64 as i64, 0xa7b0b1b2b3b4b5b6u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 17)), o);
        let o = (
            _mm_set_epi64x(0x0000a0a1a2a3a4a5u64 as i64, 0xa6a7b0b1b2b3b4b5u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 18)), o);
        let o = (
            _mm_set_epi64x(0x000000a0a1a2a3a4u64 as i64, 0xa5a6a7b0b1b2b3b4u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 19)), o);
        let o = (
            _mm_set_epi64x(0x00000000a0a1a2a3u64 as i64, 0xa4a5a6a7b0b1b2b3u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 20)), o);
        let o = (
            _mm_set_epi64x(0x0000000000a0a1a2u64 as i64, 0xa3a4a5a6a7b0b1b2u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 21)), o);
        let o = (
            _mm_set_epi64x(0x000000000000a0a1u64 as i64, 0xa2a3a4a5a6a7b0b1u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 22)), o);
        let o = (
            _mm_set_epi64x(0x00000000000000a0u64 as i64, 0xa1a2a3a4a5a6a7b0u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 23)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0xa0a1a2a3a4a5a6a7u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 24)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x00a0a1a2a3a4a5a6u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 25)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000a0a1a2a3a4a5u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 26)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x000000a0a1a2a3a4u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 27)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x00000000a0a1a2a3u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 28)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000a0a1a2u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 29)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x000000000000a0a1u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 30)), o);
        let o = (
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x00000000000000a0u64 as i64),
            _mm_set_epi64x(0x0000000000000000u64 as i64, 0x0000000000000000u64 as i64),
        );
        assert_word_eq(bshr_ssse3(i, _mm_set_epi64x(0, 31)), o);
    }
}
