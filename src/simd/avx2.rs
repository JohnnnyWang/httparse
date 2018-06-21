use ::iter::Bytes;

pub unsafe fn parse_uri_batch_32<'a>(bytes: &mut Bytes<'a>) {
    while bytes.as_ref().len() >= 32 {
        let advance = match_url_char_32_avx(bytes.as_ref());
        bytes.advance(advance);

        if advance != 32 {
            break;
        }
    }
}

#[target_feature(enable = "avx2")]
#[inline]
#[allow(non_snake_case, overflowing_literals)]
unsafe fn match_url_char_32_avx(buf: &[u8]) -> usize {
    debug_assert!(buf.len() >= 32);

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    let ptr = buf.as_ptr();

    let LSH: __m256i = _mm256_set1_epi8(0x0f);
    let URI: __m256i = _mm256_setr_epi8(
        0xb8, 0xfc, 0xf8, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc,
        0xfc, 0xfc, 0xfc, 0x7c, 0x54, 0x7c, 0xd4, 0x7c,
        0xb8, 0xfc, 0xf8, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc,
        0xfc, 0xfc, 0xfc, 0x7c, 0x54, 0x7c, 0xd4, 0x7c,
    );
    let ARF: __m256i = _mm256_setr_epi8(
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    );

    let data = _mm256_lddqu_si256(ptr as *const _);
    let rbms = _mm256_shuffle_epi8(URI, data);
    let cols = _mm256_and_si256(LSH, _mm256_srli_epi16(data, 4));
    let bits = _mm256_and_si256(_mm256_shuffle_epi8(ARF, cols), rbms);

    let v = _mm256_cmpeq_epi8(bits, _mm256_setzero_si256());
    let r = 0xffffffff_00000000 | _mm256_movemask_epi8(v) as u64;

    _tzcnt_u64(r) as usize
}

pub unsafe fn match_header_value_batch_32(bytes: &mut Bytes) {
    while bytes.as_ref().len() >= 32 {
        let advance = match_header_value_char_32_avx(bytes.as_ref());
        bytes.advance(advance);

        if advance != 32 {
            break;
        }
    }
}

#[target_feature(enable = "avx2")]
#[inline]
#[allow(non_snake_case)]
unsafe fn match_header_value_char_32_avx(buf: &[u8]) -> usize {
    debug_assert!(buf.len() >= 32);

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    let ptr = buf.as_ptr();

    // %x09 %x20-%x7e %x80-%xff
    let TAB: __m256i = _mm256_set1_epi8(0x09);
    let DEL: __m256i = _mm256_set1_epi8(0x7f);
    let LOW: __m256i = _mm256_set1_epi8(0x1f);

    let dat = _mm256_lddqu_si256(ptr as *const _);
    let low = _mm256_cmpgt_epi8(dat, LOW);
    let tab = _mm256_cmpeq_epi8(dat, TAB);
    let del = _mm256_cmpeq_epi8(dat, DEL);
    let bit = _mm256_andnot_si256(del, _mm256_or_si256(low, tab));
    let rev = _mm256_cmpeq_epi8(bit, _mm256_setzero_si256());
    let res = 0xffffffff_00000000 | _mm256_movemask_epi8(rev) as u64;

    _tzcnt_u64(res) as usize
}

