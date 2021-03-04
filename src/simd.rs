pub fn compare(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    assert!(needle.len() < 16);

    unsafe {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let a = _mm_loadu_si128(needle.as_ptr() as *const _);
        let len = needle.len() as i32;
        let chunk_width = 16;

        for (i, chunk) in haystack.chunks(chunk_width).enumerate() {
            let b = _mm_loadu_si128(chunk.as_ptr() as *const _);
            let idx = _mm_cmpestri(a, len, b, chunk_width as i32, _SIDD_CMP_EQUAL_ORDERED);
            if idx != 16 {
                return Some(idx as usize + (i * chunk_width));
            }
        }
    }

    None
}
