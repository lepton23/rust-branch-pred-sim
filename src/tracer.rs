use memmap2::MmapOptions;
use std::fs::File;

// Checks for x86_64 arch to emable SIMD parsing optimisations.
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Struct for trace entries
// u64 for *both* address and size for better alignment, even if size is only 3 decimal digits.
#[derive(Debug, Clone, Copy)]
pub struct TraceEntry {
    pub pc_addr: u64,
    pub ta_addr: u64,
    pub kind: char,
    pub direct: bool,
    pub conditional: bool,
    pub taken: bool,
}

#[inline(always)]
unsafe fn parse_char(ptr: *const u8) -> char {
    let byte = unsafe { ptr.read() };
    char::from(byte)
}

#[inline(always)]
unsafe fn parse_binary_triple(ptr: *const u8) -> (bool, bool, bool) {
    // load all 5 bytes at once
    let chunk = unsafe { (ptr.cast::<u32>()).read_unaligned() };

    let b0 = (chunk & 0xFF) as u8;
    let b1 = ((chunk >> 16) & 0xFF) as u8;
    let b2 = unsafe { ptr.add(4).read() };

    (b0 & 0x01 != 0, b1 & 0x01 != 0, b2 & 0x01 != 0)
}

// Use SIMD to parse pairs of addresses in one go!
// AVX2 is an x86 extension that allows processing of 32 bytes in parallel in 256 bit wide registers using SIMD instructions.
// Should be enabled on all the Intel CPUs in the labs.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn parse_line_pair_simd(ptr0: *const u8, ptr1: *const u8) -> (u64, u64) {
    // Load 16 hex chars into 128 bit registers for each line
    let lo = unsafe { _mm_loadu_si128(ptr0.cast::<__m128i>()) };
    let hi = unsafe { _mm_loadu_si128(ptr1.cast::<__m128i>()) };
    let input = _mm256_set_m128i(hi, lo); // combine into 256 bit register

    // Mask out high nibble of each byte
    let mask_0f = _mm256_set1_epi8(0x0f); // this is an 8 bit needle which we can AND over our 256 bit input
    let low_nibbles = _mm256_and_si256(input, mask_0f); // as so - grabbing the low nibble of each byte

    // ASCII digits: 0-9: 0x30 -> 0x39
    // ASCII letters: a-f: 0x61 -> 0x66, A-F: 0x41 -> 0x46
    // So if a byte > 0x39, it's a letter and needs a +9 offset to convert to a number
    let mask_39 = _mm256_set1_epi8(0x39u8.cast_signed()); // 8 bit needle with value of 0x39 we can run over 256 bit register
    let is_letter = _mm256_cmpgt_epi8(input, mask_39); // cmpgt returns 0xFF for bytes larger than 0x39

    // form a register with a value 9 in each byte position where there is a letter
    let nine_offset = _mm256_set1_epi8(9); // 8 bit needle with value of 9
    let letter_offset = _mm256_and_si256(is_letter, nine_offset); // AND it witht he is_letter mask to get 9 in letter positions

    // add offset to low nibbles to convert letters to numbers - digits are unaffected since their offset is 0
    let nibbles = _mm256_add_epi8(low_nibbles, letter_offset);

    // Bytes are now in this 2 lane format in the 256 bit register - example:
    // 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x09 0x0A 0x0B 0x0C 0x0F 0x02 0x0D 0x02
    // 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x09 0x0A 0x0B 0x0C 0x0F 0x02 0x0D 0x02
    // We need to pack nibbles together - need to reorder them with some shifting and masking
    let evens_shifted = _mm256_slli_epi16::<4>(nibbles); // nibbles in even positions are shifted to high nibble of each byte
    let evens_mask = _mm256_set1_epi16(0x00f0u16.cast_signed());
    let evens = _mm256_and_si256(evens_shifted, evens_mask); // mask out all other bits to get nibbles in even positions

    let odds_mask = _mm256_set1_epi16(0x0f00u16.cast_signed());
    let odds = _mm256_and_si256(nibbles, odds_mask); // mask out every even byte to get nibbles in odd positions
    let odds_shifted = _mm256_srli_epi16::<8>(odds); // shift these nibbles 8 bits right

    // OR the even and odd nibbles together to get the nibbles into the same bytes
    // now required bytes are in even bytes of each 128 bit lane
    let combined = _mm256_or_si256(evens, odds_shifted);

    // Use pshufb to unshuffle bytes
    // this zeros out the bytes we don't need and picks the useful bytes and packs them to the low end of each 128 bit lane
    let shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, -1, -1, 0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1,
        0, 2, 4, 6, 8, 10, 12, 14,
    );

    let packed = _mm256_shuffle_epi8(combined, shuffle);

    // extract each 128-bit lane and pull out low 64 bits
    let lane0 = _mm256_castsi256_si128(packed);
    let lane1 = _mm256_extracti128_si256::<1>(packed);

    // cast each lane to u64
    let result0 = _mm_cvtsi128_si64(lane0).cast_unsigned();
    let result1 = _mm_cvtsi128_si64(lane1).cast_unsigned();

    // return both results
    (result0, result1)
} // parse_line_pair_simd

pub fn return_line_no(path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let line_width = 42;
    let line_num = mmap.len() / line_width;

    Ok(line_num as u64)
}

// Main function to process tracefile - takes a closure which is called for each trace entry
// This allows for flexible processing of the tracefile without needing to load it all into memory at once.
// Uses memory mapping and SIMD optimisations for fast processing of large tracefiles.
pub fn process_trace<F>(path: &str, mut process: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(TraceEntry),
{
    let file = File::open(path)?;

    // Mmap::populate prefaults in all pages to avoid page faults during processing
    let mmap = unsafe { MmapOptions::new().populate().map(&file)? };

    // advise kernel of access pattern for better readahead
    // no need for MADV_WILLNEED since populate already faults pages in
    // MADV_SEQUENTIAL to tell kernel we will use a linear access pattern
    // - allows for more aggressive readahead and for deallocating pages after we have accessed them
    unsafe {
        libc::madvise(
            mmap.as_ptr() as *mut libc::c_void,
            mmap.len(),
            libc::MADV_SEQUENTIAL | libc::MADV_HUGEPAGE,
        );
    }

    let line_width = 42; // 41 + newline 
    let total_lines = mmap.len() / line_width;

    // process quads of lines with dual-lane AVX2
    for i in 0..total_lines {
        let offset = i * line_width;

        // prefetch several lines ahead to hide memory latency
        let prefetch_offset = offset + 8 * line_width; // 8 lines ahead
        if prefetch_offset < mmap.len() {
            unsafe {
                _mm_prefetch::<_MM_HINT_T0>(mmap.as_ptr().add(prefetch_offset) as *const i8);
            }
        }

        unsafe {
            let ptr = mmap.as_ptr().add(offset);
            let (pc_addr, ta_addr) = parse_line_pair_simd(ptr, ptr.add(17));
            let kind = parse_char(ptr.add(34));
            let (direct, conditional, taken) = parse_binary_triple(ptr.add(36));

            // println!("{:?}", str::from_utf8(std::slice::from_raw_parts(ptr, 42)));
            // println!("{pc_addr} {ta_addr} {direct} {conditional} {taken}");

            // pass entries into closure for processing through the branch predictor
            if conditional {
                process(TraceEntry {
                    pc_addr,
                    ta_addr,
                    kind,
                    direct,
                    conditional,
                    taken,
                });
            }
        }
    }

    Ok(())
} // process_trace
