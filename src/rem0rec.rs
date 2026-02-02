use crate::ut::{PAGE_SIZE, _mach_read_from_1, mach_read_from_2, ut_align_offset};

const REC_NEXT: u32 = 2;
pub const REC_N_NEW_EXTRA_BYTES: u32 = 5;
pub const _REC_N_OLD_EXTRA_BYTES: u32 = 6;
const _REC_NEW_STATUS: u32 = 3;
const _REC_NEW_STATUS_MASK: u32 = 0x7u32;
const _REC_NEW_STATUS_SHIFT: u32 = 0;

fn _rec_get_bit_field_1(page: &[u8], rec: u32, offs: u32, mask: u32, shift: u32) -> u32 {
    let byte_val: u8 = _mach_read_from_1(&page[(rec - offs) as usize..]);
    (byte_val as u32 & mask) >> shift
}

fn _rec_get_status(page: &[u8], rec: u32) -> u32 {
    _rec_get_bit_field_1(
        page,
        rec,
        _REC_NEW_STATUS,
        _REC_NEW_STATUS_MASK,
        _REC_NEW_STATUS_SHIFT,
    )
}

pub fn rec_get_next_offs(page: &[u8], rec: u32, comp: u32) -> u32 {
    let field_val: u32 = mach_read_from_2(&page[(rec - REC_NEXT) as usize..]) as u32;
    if comp > 0 {
        if field_val == 0 {
            return 0;
        }

        eprintln!("Record may not be in the same page");
        return ut_align_offset(rec + field_val, PAGE_SIZE as u32);
    } else {
        return field_val;
    }
}
