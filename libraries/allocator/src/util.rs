/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if !is_power_of_two(align) {
        panic!("align is not a power of 2");
    }

    addr / align * align
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    let aligned_down = align_down(addr, align);
    if aligned_down == addr {
        aligned_down
    } else {
        aligned_down + align
    }
}

pub fn is_power_of_two(align: usize) -> bool {
    if align == 0 {
        return false;
    }

    let bit_count = 8 * core::mem::size_of::<usize>();
    let mut first_bit = 0;

    for i in 0..bit_count {
        if align & (0b1 << i) > 0 {
            first_bit = i;
            break;
        }
    }

    (align ^ (0b1 << first_bit)) == 0
}