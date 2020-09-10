
def printtest(count):
    x = (0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7 << (count*8)) % (2**256)
    d = x & 0xffffffffffffffff
    c = (x>>64) & 0xffffffffffffffff
    b = (x>>128) & 0xffffffffffffffff
    a = (x>>192) & 0xffffffffffffffff
    print("        let o = (_mm_set_epi64x(0x{:016x}u64 as i64, 0x{:016x}u64 as i64), _mm_set_epi64x(0x{:016x}u64 as i64, 0x{:016x}u64 as i64));".format(c, d, a, b))
    print("        assert_word_eq(bshl_sse2(i, _mm_set_epi64x(0, {})), o);".format(count))
    #print("")

def printtest2(count):
    x = (0xa0a1a2a3a4a5a6a7b0b1b2b3b4b5b6b7c0c1c2c3c4c5c6c7d0d1d2d3d4d5d6d7 >> (count*8)) % (2**256)
    d = x & 0xffffffffffffffff
    c = (x>>64) & 0xffffffffffffffff
    b = (x>>128) & 0xffffffffffffffff
    a = (x>>192) & 0xffffffffffffffff
    print("        let o = (_mm_set_epi64x(0x{:016x}u64 as i64, 0x{:016x}u64 as i64), _mm_set_epi64x(0x{:016x}u64 as i64, 0x{:016x}u64 as i64));".format(c, d, a, b))
    print("        assert_word_eq(bshr_sse2(i, _mm_set_epi64x(0, {})), o);".format(count))
    #print("")

for i in range(0, 32):
  printtest2(i)

# 0xd0d1d2d3d4d5d6d7 0xc0c1c2c3c4c5c6c7 0xb0b1b2b3b4b5b6b7 0xa0a1a2a3a4a5a6a

#  let i = (_mm_set_epi64x(0xc0c1c2c3c4c5c6c7u64 as i64, 0xd0d1d2d3d4d5d6d7u64 as i64), _mm_set_epi64x(0xa0a1a2a3a4a5a6a7u64 as i64, 0xb0b1b2b3b4b5b6b7u64 as i64));